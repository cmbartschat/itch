use git2::{Commit, ErrorCode};

use crate::{commit::count_commits_since, ctx::Ctx, error::Attempt};

pub fn list_command(ctx: &Ctx) -> Attempt {
    let (selected_color, muted_color, clear_color) = if ctx.color_enabled() {
        ("\x1b[1;34m", "\x1b[1;30m", "\x1b[0m")
    } else {
        ("", "", "")
    };
    let (selected_prefix, normal_prefix) = if ctx.is_pipe() {
        ("", "")
    } else {
        ("> ", "  ")
    };

    let include_status = !ctx.is_pipe();
    let mut main_commit: Option<Commit> = None;
    for branch in ctx.repo.branches(Some(git2::BranchType::Local))? {
        match branch {
            Ok(b) => match b.0.name() {
                Ok(Some(name)) => {
                    if b.0.is_head() {
                        print!("{selected_color}{selected_prefix}{name}{clear_color}");
                    } else {
                        print!("{normal_prefix}{name}");
                    }

                    if include_status {
                        let commit = if let Some(e) = main_commit.as_ref() {
                            e
                        } else {
                            let main = ctx.repo.find_branch("main", git2::BranchType::Local)?;
                            main_commit = Some(main.into_reference().peel_to_commit()?);
                            main_commit.as_ref().unwrap()
                        };

                        match ctx
                            .repo
                            .merge_base(commit.id(), b.0.into_reference().peel_to_commit()?.id())
                        {
                            Ok(fork_id) => {
                                let fork_commit = ctx.repo.find_commit(fork_id)?;

                                let behind = count_commits_since(ctx, &fork_commit, commit)?;

                                if behind > 0 {
                                    println!("{muted_color} {behind} behind{clear_color}");
                                } else {
                                    println!();
                                }
                            }
                            Err(e) => match e.code() {
                                ErrorCode::NotFound => {
                                    println!("{selected_color} orphan{clear_color}");
                                }
                                _ => {
                                    println!(
                                        "{selected_color} error calculating status{clear_color}"
                                    );
                                }
                            },
                        };
                    } else {
                        println!();
                    }
                }
                _ => println!("<Invalid branch>"),
            },
            _ => println!("<Invalid branch>"),
        }
    }

    Ok(())
}
