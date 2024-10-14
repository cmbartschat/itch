use std::path::Path;

use git2::{build::TreeUpdateBuilder, Commit, FileMode, ResetType, Tree};

use crate::{cli::UnsaveArgs, ctx::Ctx, error::Attempt};

pub fn unsave_command(ctx: &Ctx, args: &UnsaveArgs) -> Attempt {
    println!("You want to unsave: {args:?}");
    // return Ok(());
    let head_commit = ctx.repo.head()?.peel_to_commit()?;
    let base_commit = ctx
        .repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference()
        .peel_to_commit()?;

    let fork_commit = ctx
        .repo
        .find_commit(ctx.repo.merge_base(base_commit.id(), head_commit.id())?)?;

    if !args.args.is_empty() {
        let current_tree = head_commit.tree()?;
        let prev_commit = head_commit.parent(0)?;
        let prev_tree = prev_commit.tree()?;

        let mut new_tree_builder = TreeUpdateBuilder::new();

        for file_path in args.args.iter().map(Path::new) {
            // if (current_tree.get_path(file_path)
            // new_tree_builder.remove(file_path);

            match prev_tree.get_path(file_path) {
                Ok(entry) => {
                    new_tree_builder.upsert(file_path, entry.id(), FileMode::Tree);
                }
                Err(e) => match e.code() {
                    git2::ErrorCode::NotFound => {
                        let should_delete = match current_tree.get_path(file_path) {
                            Ok(_) => true,
                            Err(e) => {
                                if e.code() == git2::ErrorCode::NotFound {
                                    false
                                } else {
                                    return Err(e);
                                }
                            }
                        };
                        if should_delete {
                            new_tree_builder.remove(file_path);
                        }
                    }
                    _ => return Err(e),
                },
            }
        }

        // TODO: Undo save if they're all reset
        // handle sub-paths

        let new_tree: Tree = ctx
            .repo
            .find_tree(new_tree_builder.create_updated(&ctx.repo, &prev_tree)?)?;
        let parents: Vec<Commit> = head_commit.parents().collect();
        let parent_refs: Vec<&Commit> = parents.iter().collect();

        let committed = ctx.repo.commit(
            None,
            &head_commit.author(),
            &head_commit.committer(),
            head_commit.message().unwrap_or(""),
            &new_tree,
            &parent_refs,
        )?;

        println!("Committed: {committed:?}");

        ctx.repo.reset(
            &ctx.repo.find_object(committed, None)?,
            ResetType::Mixed,
            None,
        )?;

        return Ok(());
    }

    ctx.repo
        .reset(&fork_commit.into_object(), ResetType::Mixed, None)?;

    Ok(())
}
