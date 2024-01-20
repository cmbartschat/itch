use std::{
    io::{self, Write},
    ops::BitAnd,
};

use git2::{
    Error, ErrorCode, Index, IndexConflict, IndexEntry, Oid, RebaseOperationType, RebaseOptions,
    Repository,
};

use crate::{
    cli::SaveArgs,
    command::save::save_command,
    consts::TEMP_COMMIT_PREFIX,
    ctx::Ctx,
    path::bytes2path,
    reset::pop_and_reset,
    sync::{
        Conflict, FullSyncArgs, MergeConflict, ResolutionChoice, ResolutionMap, SyncDetails,
        SyncResult,
    },
};

fn yes_or_no(prompt: &str, by_default: Option<bool>) -> bool {
    print!("{prompt} ");

    match by_default {
        Some(true) => print!("Y/n "),
        Some(false) => print!("y/N "),
        None => print!("y/n "),
    };

    loop {
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match (input.trim(), by_default) {
            ("y", _) => return true,
            ("n", _) => return false,
            ("", Some(r)) => return r,
            _ => {
                println!("Unrecognized option. Try again:");
            }
        }
    }
}

fn entry_without_conflicts(mut entry: IndexEntry) -> IndexEntry {
    entry.flags = entry.flags.bitand(0x3000_u16.reverse_bits());
    entry
}

fn resolve_conflict(
    ctx: &Ctx,
    index: &mut Index,
    conflict: IndexConflict,
    resolutions: Option<&ResolutionMap>,
) -> Result<Option<Conflict>, Error> {
    let repo = &ctx.repo;
    match (conflict.their, conflict.our) {
        (Some(branch_entry), Some(main_entry)) => {
            let current_path = bytes2path(&branch_entry.path)?;
            let main_path = bytes2path(&main_entry.path)?;
            if let Some(resolution) =
                resolutions.and_then(|f| f.get(current_path.to_string_lossy().as_ref()))
            {
                match resolution {
                    ResolutionChoice::Yours => {
                        index.add(&entry_without_conflicts(branch_entry))?;
                        return Ok(None);
                    }
                    ResolutionChoice::Theirs => {
                        index.add(&entry_without_conflicts(main_entry))?;
                        return Ok(None);
                    }
                    ResolutionChoice::Manual(str) => {
                        let mut new_entry = IndexEntry::from(branch_entry);
                        new_entry.id = repo.blob(str.as_bytes())?;
                        index.add(&entry_without_conflicts(new_entry))?;
                        return Ok(None);
                    }
                }
            }

            if (!ctx.can_prompt()) {
                let main_file: String = main_path.to_string_lossy().into();
                let branch_file: String = current_path.to_string_lossy().into();
                let main_blob = repo.find_blob(main_entry.id)?;
                let branch_blob = repo.find_blob(branch_entry.id)?;

                match (main_blob.is_binary(), branch_blob.is_binary()) {
                    (false, false) => {
                        return Ok(Some(Conflict::Merge(MergeConflict {
                            main_path: main_file,
                            branch_path: branch_file,
                            main_content: String::from_utf8_lossy(main_blob.content()).into(),
                            branch_content: String::from_utf8_lossy(branch_blob.content()).into(),
                            merge_content: String::from(""),
                        })));
                    }
                    _ => {
                        return Ok(Some(Conflict::OpaqueMerge(main_file, branch_file)));
                    }
                }
            }

            let prompt = format!(
                "{} is conflicted. Keep your changes?",
                current_path.to_string_lossy(),
            );

            if yes_or_no(&prompt, None) {
                index.remove_path(&main_path)?;
                index.add(&entry_without_conflicts(branch_entry))?;
            } else {
                index.remove_path(&current_path)?;
                index.add(&entry_without_conflicts(main_entry))?;
            };
        }
        // File deleted on main
        (Some(branch_entry), None) => {
            let current_path = bytes2path(&branch_entry.path)?;
            if let Some(resolution) =
                resolutions.and_then(|f| f.get(current_path.to_string_lossy().as_ref()))
            {
                match resolution {
                    ResolutionChoice::Yours => {
                        index.add(&entry_without_conflicts(branch_entry))?;
                        return Ok(None);
                    }
                    ResolutionChoice::Theirs => {
                        index.remove_path(&current_path)?;
                        return Ok(None);
                    }
                    _ => todo!("More types of resolution"),
                }
            }

            if (!ctx.can_prompt()) {
                return Ok(Some(Conflict::MainDeletion(
                    current_path.to_string_lossy().into(),
                )));
            }

            let prompt = format!(
                "{} was deleted on main. Keep your changes?",
                current_path.to_string_lossy()
            );
            if yes_or_no(&prompt, Some(true)) {
                index.add(&entry_without_conflicts(branch_entry))?;
            } else {
                index.remove_path(&current_path)?;
            }
        }
        // File deleted on branch
        (None, Some(main_entry)) => {
            let current_path = bytes2path(&main_entry.path)?;
            if let Some(resolution) =
                resolutions.and_then(|f| f.get(current_path.to_string_lossy().as_ref()))
            {
                match resolution {
                    ResolutionChoice::Yours => {
                        index.remove_path(&current_path)?;
                        return Ok(None);
                    }
                    ResolutionChoice::Theirs => {
                        index.add(&entry_without_conflicts(main_entry))?;
                        return Ok(None);
                    }
                    _ => todo!("More types of resolution"),
                }
            }

            if (!ctx.can_prompt()) {
                return Ok(Some(Conflict::BranchDeletion(
                    current_path.to_string_lossy().into(),
                )));
            }

            let prompt = format!(
                "{} was modified on main. Keep your delete?",
                current_path.to_string_lossy()
            );
            if yes_or_no(&prompt, Some(true)) {
                index.remove_path(&current_path)?;
            } else {
                index.add(&entry_without_conflicts(main_entry))?;
            }
        }
        (None, None) => {}
    };

    Ok(None)
}

fn sync_branch(
    ctx: &Ctx,
    branch_name: &str,
    resolutions: Option<&ResolutionMap>,
) -> Result<SyncDetails, Error> {
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
                    let conflicts = index.conflicts()?;
                    println!("\nWe have some conflicts to resolve: {}", conflicts.count());

                    rebase.inmemory_index()?.conflicts()?.try_for_each(
                        |conflict| -> Result<(), Error> {
                            if let Some(conflict) =
                                resolve_conflict(ctx, &mut index, conflict?, resolutions)?
                            {
                                details.push(conflict);
                            }
                            Ok(())
                        },
                    )?;
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

pub fn sync_command(ctx: &Ctx, args: &FullSyncArgs) -> Result<SyncResult, Error> {
    let mut results: SyncResult = vec![];

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

    if args.names.len() == 0 {
        let repo_head = ctx.repo.head()?;
        let head_name_str = repo_head.name().unwrap();
        let head_name = head_name_str[head_name_str.rfind("/").map_or(0, |e| e + 1)..].to_owned();
        results.push(sync_branch(&ctx, &head_name, args.resolutions.get(0))?);
    }

    for (index, branch) in args.names.iter().enumerate() {
        results.push(sync_branch(&ctx, &branch, args.resolutions.get(index))?);
    }

    pop_and_reset(ctx)?;

    Ok(results)
}
