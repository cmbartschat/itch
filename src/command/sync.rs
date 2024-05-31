use std::{
    collections::HashMap,
    io::{self, Write},
    ops::BitAnd,
    path::PathBuf,
};

use git2::{
    Error, ErrorCode, Index, IndexConflict, IndexEntry, Oid, RebaseOperationType, RebaseOptions,
};

use crate::{
    cli::{SaveArgs, SyncArgs},
    command::save::save_command,
    consts::TEMP_COMMIT_PREFIX,
    ctx::Ctx,
    diff::get_merge_text,
    editor::edit_temp_text,
    path::bytes2path,
    remote::pull_main,
    reset::pop_and_reset,
};

fn ask_option(prompt: &str, options: &[&str], default: Option<&str>) -> String {
    print!("{prompt} ");

    let last_index = options.len() - 1;
    let second_to_last_index = options.len() - 2;

    let mut shortcut_map: HashMap<String, String> = HashMap::new();
    let mut fullform_map: HashMap<String, String> = HashMap::new();

    if let Some(default) = default {
        shortcut_map.insert("".into(), default.into());
    }

    options.iter().for_each(|f| {
        for i in 1..f.len() {
            let possible_shortcut = &f[0..i];
            if !shortcut_map.contains_key(possible_shortcut) {
                shortcut_map.insert(possible_shortcut.into(), f.to_string());
                fullform_map.insert(f.to_string(), possible_shortcut.into());
                break;
            }
        }
    });

    for (index, option) in options.iter().enumerate() {
        if default == Some(option) {
            print!("{} (default)", option.to_string());
        } else if let Some(shortcut) = fullform_map.get(*option) {
            print!("({}){}", shortcut, &option[shortcut.len()..]);
        }
        if index != last_index {
            if last_index == 1 {
                print!(" ");
            } else {
                print!(", ");
            }
        }
        if index == second_to_last_index {
            print!("or ");
        }
    }

    print!(": ");

    loop {
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if options.contains(&&input.as_str()) {
            return input;
        }
        if let Some(input) = shortcut_map.get_mut(&input) {
            return input.clone();
        }
        println!("Unrecognized option. Try again:");
    }
}

fn delete_entry(index: &mut Index, path: &PathBuf) -> Result<(), Error> {
    index.remove_path(path)
}

fn select_entry(index: &mut Index, old_path: &PathBuf, mut entry: IndexEntry) -> Result<(), Error> {
    index.remove_path(old_path)?;
    entry.flags = entry.flags.bitand(0x3000_u16.reverse_bits());
    index.add(&entry)
}

fn resolve_conflict(ctx: &Ctx, index: &mut Index, conflict: IndexConflict) -> Result<(), Error> {
    if !ctx.can_prompt() {
        return Err(Error::from_str(
            "Cannot resolve conflict without user input.",
        ));
    }
    let repo = &ctx.repo;
    match (conflict.their, conflict.our) {
        (Some(branch_entry), Some(main_entry)) => {
            let current_path = bytes2path(&branch_entry.path)?;
            let main_path = bytes2path(&main_entry.path)?;

            let prompt = format!(
                "{} is conflicted. What would you like to do?",
                current_path.to_string_lossy(),
            );

            let options = ["keep", "reset", "edit"];

            match ask_option(&prompt, &options, Some("edit")).as_str() {
                "keep" => select_entry(index, &main_path, branch_entry),
                "reset" => select_entry(index, &current_path, main_entry),
                "edit" => {
                    let path = bytes2path(&branch_entry.path)?;

                    let original_id = conflict
                        .ancestor
                        .map(|e| e.id)
                        .unwrap_or_else(|| repo.blob("".as_bytes()).unwrap());
                    let patch_text =
                        get_merge_text(&ctx.repo, &original_id, &main_entry.id, &branch_entry.id)?;
                    let edited_string = edit_temp_text(&patch_text, path.extension())?;
                    index.remove_path(&main_path)?;
                    let mut new_entry = IndexEntry::from(branch_entry);
                    new_entry.id = repo.blob(edited_string.as_bytes())?;
                    select_entry(index, &main_path, new_entry)
                }
                _ => panic!("Unhandled option"),
            }
        }
        // File deleted on main
        (Some(branch_entry), None) => {
            let current_path = bytes2path(&branch_entry.path)?;
            match ask_option(
                &format!(
                    "{} was deleted on main. What would you like to do?",
                    current_path.to_string_lossy(),
                ),
                &["delete", "keep"],
                Some("keep"),
            )
            .as_str()
            {
                "keep" => select_entry(index, &current_path, branch_entry),
                "delete" => delete_entry(index, &current_path),
                _ => panic!("Unhandled option"),
            }
        }
        // File deleted on branch
        (None, Some(main_entry)) => {
            let current_path = bytes2path(&main_entry.path)?;
            match ask_option(
                &format!(
                    "{} was deleted, but has been modified on main. What would you like to do?",
                    current_path.to_string_lossy(),
                ),
                &["delete", "keep"],
                Some("keep"),
            )
            .as_str()
            {
                "delete" => delete_entry(index, &current_path),
                "keep" => select_entry(index, &current_path, main_entry),
                _ => panic!("Unhandled option"),
            }
        }
        (None, None) => Ok(()),
    }
}

fn sync_branch(ctx: &Ctx, branch_name: &str) -> Result<(), Error> {
    let repo = &ctx.repo;
    let branch_ref = repo
        .find_branch(&branch_name, git2::BranchType::Local)?
        .into_reference();
    let main_ref = repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference();

    let upstream_id = repo.reference_to_annotated_commit(&main_ref)?;
    let branch_id = repo.reference_to_annotated_commit(&branch_ref)?;

    let mut rebase = repo.rebase(
        Some(&branch_id),
        Some(&upstream_id),
        None,
        Some(&mut RebaseOptions::new().inmemory(true)),
    )?;

    let mut final_id: Oid = upstream_id.id();

    while let Some(Ok(operation)) = rebase.next() {
        match operation.kind() {
            Some(RebaseOperationType::Pick) => {
                let mut index = rebase.inmemory_index()?;
                if index.has_conflicts() {
                    let mut conflicts: Vec<IndexConflict> = vec![];

                    rebase.inmemory_index()?.conflicts()?.try_for_each(
                        |c| -> Result<(), Error> {
                            conflicts.push(c?);
                            Ok(())
                        },
                    )?;

                    println!(
                        "\nWe have {} {} to resolve",
                        conflicts.len(),
                        if conflicts.len() == 1 {
                            "conflict"
                        } else {
                            "conflicts"
                        }
                    );

                    conflicts
                        .into_iter()
                        .try_for_each(|conflict| resolve_conflict(ctx, &mut index, conflict))?;
                }
            }
            Some(RebaseOperationType::Fixup) => {
                // Ok, whatever
                todo!("Handle fixup");
            }
            Some(RebaseOperationType::Edit) => {
                todo!("Handle edit");
            }
            _ => {
                todo!("Handle: {:?}", operation);
            }
        };

        match rebase.commit(None, &repo.signature()?, None) {
            Ok(id) => {
                final_id = id;
            }
            Err(e) => {
                if e.code() != ErrorCode::Applied {
                    return Err(e);
                }
            }
        }
    }

    rebase.finish(Some(&repo.signature()?))?;

    let final_commit = repo.find_commit(final_id)?;
    if repo
        .find_branch(branch_name, git2::BranchType::Local)?
        .is_head()
    {
        repo.reset(final_commit.as_object(), git2::ResetType::Hard, None)?;
    } else {
        repo.branch(&branch_name, &final_commit, true)?;
    }

    Ok(())
}

pub fn sync_command(ctx: &Ctx, args: &SyncArgs) -> Result<(), Error> {
    save_command(
        ctx,
        &SaveArgs {
            message: vec![
                TEMP_COMMIT_PREFIX.to_string(),
                "Save before sync".to_owned(),
            ],
        },
        true,
    )?;

    match pull_main(ctx) {
        Err(e) => println!("Skipping pull from remote due to: {}", e.message()),
        _ => {}
    }

    if args.names.len() == 0 {
        let repo_head = ctx.repo.head()?;
        let head_name_str = repo_head.name().unwrap();
        let head_name = head_name_str[head_name_str.rfind("/").map_or(0, |e| e + 1)..].to_owned();
        sync_branch(ctx, &head_name)?;
    }
    for branch in &args.names {
        sync_branch(ctx, &branch)?;
    }

    pop_and_reset(ctx)
}
