use gix::Commit;

use crate::{branch::find_main, commit::count_commits_since, ctx::Ctx, error::Attempt};

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
    let head_ref = ctx.repo.head_ref()?;
    for branch in ctx.repo.references()?.local_branches()? {
        match branch {
            Ok(mut b) => {
                let name = b.name().shorten().to_string();
                if head_ref.as_ref().is_some_and(|f| f.name() == b.name()) {
                    print!("{selected_color}{selected_prefix}{name}{clear_color}");
                } else {
                    print!("{normal_prefix}{name}");
                }

                if include_status {
                    let commit = if let Some(e) = main_commit.as_ref() {
                        e
                    } else {
                        main_commit = Some(find_main(ctx)?.peel_to_commit()?);
                        main_commit.as_ref().unwrap()
                    };

                    match ctx.repo.merge_base(commit.id(), b.peel_to_commit()?.id()) {
                        Ok(fork_id) => {
                            let fork_commit = ctx.repo.find_commit(fork_id)?;

                            let behind = count_commits_since(ctx, &fork_commit, commit)?;

                            if behind > 0 {
                                println!("{muted_color} {behind} behind{clear_color}");
                            } else {
                                println!();
                            }
                        }
                        Err(e) => match e {
                            gix::repository::merge_base::Error::NotFound { .. } => {
                                println!("{selected_color} orphan{clear_color}");
                            }
                            _ => {
                                println!("{selected_color} error calculating status{clear_color}");
                            }
                        },
                    }
                } else {
                    println!();
                }
            }
            _ => println!("<Invalid branch>"),
        }
    }

    Ok(())
}
