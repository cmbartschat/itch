use std::{
    io::{self, Write},
    ops::BitAnd,
};

use git2::{Error, ErrorCode, Oid, RebaseOperationType, RebaseOptions, Repository};

use crate::{cli::SyncArgs, ctx::Ctx, path::bytes2path};

fn sync_branch(repo: &Repository, branch_name: &str) -> Result<(), Error> {
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

    let mut final_id: Option<Oid> = None;

    if rebase.len() == 0 {
        final_id = Some(upstream_id.id());
    } else {
        while let Some(Ok(operation)) = rebase.next() {
            match operation.kind() {
                Some(RebaseOperationType::Pick) => {
                    let mut index = rebase.inmemory_index()?;
                    if index.has_conflicts() {
                        let conflicts = index.conflicts()?;
                        println!("\nWe have some conflicts to resolve: {}", conflicts.count());

                        rebase.inmemory_index()?.conflicts()?.try_for_each(
                            |conflict| -> Result<(), Error> {
                                let conflict = conflict?;
                                let Some(ours) = conflict.our else {
                                    return Err(Error::from_str(
                                        "Expected conflict to contain 'our'",
                                    ));
                                };
                                let Some(theirs) = conflict.their else {
                                    return Err(Error::from_str(
                                        "Expected conflict to contain 'their'",
                                    ));
                                };

                                let current_path = bytes2path(&ours.path)?;

                                if current_path != bytes2path(&theirs.path)? {
                                    return Err(Error::from_str(
                                        "Conflict contains two different files.",
                                    ));
                                }

                                println!(
                                    "File: {} is conflicted. Keep yours? y/n ",
                                    current_path.to_string_lossy()
                                );
                                let keep_yours = loop {
                                    io::stdout().flush().unwrap();
                                    let mut input = String::new();
                                    io::stdin().read_line(&mut input).unwrap();
                                    if input.trim() == "y" {
                                        break true;
                                    } else if input.trim() == "n" {
                                        break false;
                                    }
                                    println!("Unknown command. Try again:");
                                };

                                index.remove_path(&current_path)?;

                                let mut final_entry = if keep_yours { theirs } else { ours };

                                final_entry.flags =
                                    final_entry.flags.bitand(0x3000_u16.reverse_bits());

                                index.add(&final_entry)?;

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

            match rebase.commit(None, &repo.signature()?, None) {
                Ok(id) => {
                    final_id = Some(id);
                }
                Err(e) => {
                    if e.code() != ErrorCode::Applied {
                        return Err(e);
                    }
                }
            }
        }
    }

    rebase.finish(Some(&repo.signature()?))?;

    match final_id {
        Some(id) => {
            let final_commit = repo.find_commit(id)?;
            if repo
                .find_branch(branch_name, git2::BranchType::Local)?
                .is_head()
            {
                repo.reset(final_commit.as_object(), git2::ResetType::Hard, None)?;
            } else {
                repo.branch(&branch_name, &final_commit, true)?;
            }
        }
        _ => {}
    }

    Ok(())
}

pub fn sync_command(ctx: &Ctx, args: &SyncArgs) -> Result<(), Error> {
    if args.names.len() == 0 {
        let repo_head = ctx.repo.head()?;
        let head_name_str = repo_head.name().unwrap();
        let head_name = head_name_str[head_name_str.rfind("/").map_or(0, |e| e + 1)..].to_owned();
        return sync_branch(&ctx.repo, &head_name);
    }
    for branch in &args.names {
        sync_branch(&ctx.repo, &branch)?;
    }

    Ok(())
}
