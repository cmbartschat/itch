use std::{
    collections::HashMap,
    io::{self, Write},
    ops::BitAnd,
    path::PathBuf,
};

use git2::{
    ErrorCode, Index, IndexConflict, IndexEntry, Oid, RebaseOperationType, RebaseOptions,
    Repository,
};

use crate::{
    branch::get_head_name,
    cli::SyncArgs,
    ctx::Ctx,
    diff::get_merge_text,
    editor::edit_temp_text,
    error::{fail, Attempt, Maybe},
    path::bytes2path,
    remote::pull_main,
    reset::pop_and_reset,
    save::save_temp,
    sync::{Conflict, MergeConflict, ResolutionChoice, ResolutionMap, SyncDetails},
};

fn ask_option(prompt: &str, options: &[&str], default: Option<&str>) -> String {
    eprint!("{prompt} ");

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
            eprint!("{} (default)", option.to_string());
        } else if let Some(shortcut) = fullform_map.get(*option) {
            eprint!("({}){}", shortcut, &option[shortcut.len()..]);
        }
        if index != last_index {
            if last_index == 1 {
                eprint!(" ");
            } else {
                eprint!(", ");
            }
        }
        if index == second_to_last_index {
            eprint!("or ");
        }
    }

    eprint!(": ");

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
        eprintln!("Unrecognized option. Try again:");
    }
}

fn delete_entry(index: &mut Index, path: &PathBuf) -> Attempt {
    index.remove_path(path)
}

fn clone_entry(entry: &IndexEntry) -> IndexEntry {
    IndexEntry {
        ctime: entry.ctime,
        mtime: entry.mtime,
        dev: entry.dev,
        ino: entry.ino,
        mode: entry.mode,
        uid: entry.uid,
        gid: entry.gid,
        file_size: entry.file_size,
        id: entry.id,
        flags: entry.flags,
        flags_extended: entry.flags_extended,
        path: entry.path.clone(),
    }
}

fn select_entry(index: &mut Index, old_path: &PathBuf, entry: &IndexEntry) -> Attempt {
    index.remove_path(old_path)?;
    let mut new_entry = clone_entry(entry);
    new_entry.flags = new_entry.flags.bitand(0x3000_u16.reverse_bits());
    index.add(&new_entry)
}

fn extract_path(conflict: &IndexConflict) -> Maybe<PathBuf> {
    if let Some(our) = conflict.our.as_ref() {
        bytes2path(&our.path)
    } else {
        bytes2path(&conflict.ancestor.as_ref().unwrap().path)
    }
}

fn apply_resolution(
    repo: &Repository,
    index: &mut Index,
    conflict: &IndexConflict,
    resolution: &ResolutionChoice,
) -> Attempt {
    let current_path = extract_path(&conflict)?;

    match (resolution, conflict.our.as_ref(), conflict.their.as_ref()) {
        (ResolutionChoice::Incoming, _, None) => delete_entry(index, &current_path),
        (ResolutionChoice::Incoming, _, Some(choice)) => select_entry(index, &current_path, choice),
        (ResolutionChoice::Base, None, _) => delete_entry(index, &current_path),
        (ResolutionChoice::Base, Some(choice), _) => select_entry(index, &current_path, choice),
        (ResolutionChoice::Manual(str), _, _) => {
            let mut new_entry = clone_entry(conflict.their.as_ref().unwrap());
            new_entry.id = repo.blob(str.as_bytes())?;
            select_entry(index, &current_path, &new_entry)
        }
    }
}

fn resolve_conflict(
    ctx: &Ctx,
    index: &mut Index,
    conflict: &IndexConflict,
    resolutions: Option<&ResolutionMap>,
) -> Maybe<Option<Conflict>> {
    let repo = &ctx.repo;
    let current_path = extract_path(&conflict)?;
    let current_path_string: String = current_path.to_string_lossy().into();

    if let Some(resolution) = resolutions.and_then(|f| f.get(&current_path_string)) {
        apply_resolution(repo, index, conflict, resolution)?;
        return Ok(None);
    }

    let resolution = match (&conflict.their, &conflict.our) {
        (Some(branch_entry), Some(main_entry)) => {
            if !ctx.can_prompt() {
                let main_blob = repo.find_blob(main_entry.id)?;
                let branch_blob = repo.find_blob(branch_entry.id)?;

                let original_id = if let Some(ancestor_entry) = conflict.ancestor.as_ref() {
                    ancestor_entry.id
                } else {
                    repo.blob("".as_bytes())?
                };

                match (main_blob.is_binary(), branch_blob.is_binary()) {
                    (false, false) => {
                        return Ok(Some(Conflict::Merge(MergeConflict {
                            path: current_path_string,
                            main_content: String::from_utf8_lossy(main_blob.content()).into(),
                            branch_content: String::from_utf8_lossy(branch_blob.content()).into(),
                            merge_content: get_merge_text(
                                repo,
                                &original_id,
                                &main_entry.id,
                                &branch_entry.id,
                            )?,
                        })));
                    }
                    _ => {
                        return Ok(Some(Conflict::OpaqueMerge(current_path_string)));
                    }
                }
            }

            let prompt = format!(
                "{} is conflicted. What would you like to do?",
                current_path_string,
            );

            let options = ["keep", "reset", "edit"];

            match ask_option(&prompt, &options, Some("edit")).as_str() {
                "keep" => ResolutionChoice::Incoming,
                "reset" => ResolutionChoice::Base,
                "edit" => {
                    let path = bytes2path(&branch_entry.path)?;

                    let original_id = conflict
                        .ancestor
                        .as_ref()
                        .map(|e| e.id)
                        .unwrap_or_else(|| repo.blob("".as_bytes()).unwrap());
                    let patch_text =
                        get_merge_text(&ctx.repo, &original_id, &main_entry.id, &branch_entry.id)?;
                    let edited_string = edit_temp_text(&patch_text, path.extension())?;
                    ResolutionChoice::Manual(edited_string)
                }
                _ => panic!("Unhandled option"),
            }
        }
        // File deleted on main
        (Some(branch_entry), None) => {
            let current_path = bytes2path(&branch_entry.path)?;
            if !ctx.can_prompt() {
                return Ok(Some(Conflict::MainDeletion(current_path_string)));
            }
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
                "keep" => ResolutionChoice::Incoming,
                "delete" => ResolutionChoice::Base,
                _ => panic!("Unhandled option"),
            }
        }
        // File deleted on branch
        (None, Some(_)) => {
            if !ctx.can_prompt() {
                return Ok(Some(Conflict::BranchDeletion(current_path_string)));
            }

            match ask_option(
                &format!(
                    "{} was deleted, but has been modified on main. What would you like to do?",
                    current_path_string,
                ),
                &["delete", "keep"],
                Some("keep"),
            )
            .as_str()
            {
                "delete" => ResolutionChoice::Incoming,
                "keep" => ResolutionChoice::Base,
                _ => panic!("Unhandled option"),
            }
        }
        (None, None) => panic!("Expected either main or branch entry"),
    };

    apply_resolution(repo, index, conflict, &resolution)?;

    Ok(None)
}

pub fn try_sync_branch(
    ctx: &Ctx,
    branch_name: &str,
    resolutions: Option<&ResolutionMap>,
) -> Maybe<SyncDetails> {
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

    let mut details: Vec<Conflict> = vec![];

    while let Some(Ok(operation)) = rebase.next() {
        match operation.kind() {
            Some(RebaseOperationType::Pick) => {
                let mut index = rebase.inmemory_index()?;
                if index.has_conflicts() {
                    let mut conflicts: Vec<IndexConflict> = vec![];

                    rebase
                        .inmemory_index()?
                        .conflicts()?
                        .try_for_each(|c| -> Attempt {
                            conflicts.push(c?);
                            Ok(())
                        })?;

                    eprintln!(
                        "\nThere are {} {} to resolve.",
                        conflicts.len(),
                        if conflicts.len() == 1 {
                            "conflict"
                        } else {
                            "conflicts"
                        }
                    );

                    conflicts.into_iter().try_for_each(|conflict| -> Attempt {
                        if let Some(r) = resolve_conflict(ctx, &mut index, &conflict, resolutions)?
                        {
                            details.push(r);
                        }
                        Ok(())
                    })?;
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

        if details.len() > 0 {
            return Ok(SyncDetails::Conflicted(details));
        }

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

    Ok(SyncDetails::Complete)
}

fn sync_branch(ctx: &Ctx, branch_name: &str) -> Attempt {
    match try_sync_branch(ctx, branch_name, None)? {
        SyncDetails::Complete => Ok(()),
        SyncDetails::Conflicted(_) => fail("Still conflicted after sync."),
    }
}

pub fn sync_command(ctx: &Ctx, args: &SyncArgs) -> Attempt {
    save_temp(ctx)?;

    match pull_main(ctx) {
        Err(e) => eprintln!("Skipping pull from remote due to: {}", e.message()),
        _ => {}
    }

    if args.names.len() == 0 {
        sync_branch(ctx, &get_head_name(ctx)?)?;
    }
    for branch in &args.names {
        sync_branch(ctx, &branch)?;
    }

    pop_and_reset(ctx)
}
