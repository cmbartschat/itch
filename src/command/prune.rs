use git2::BranchType;

use crate::{cli::DeleteArgs, command::delete::delete_command, ctx::Ctx, error::Attempt};

pub fn prune_command(ctx: &Ctx) -> Attempt {
    let mut branches_to_delete: Vec<String> = vec![];

    let main_id = ctx
        .repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference()
        .peel_to_commit()?
        .id();

    for branch in ctx.repo.branches(Some(git2::BranchType::Local))? {
        if let Ok((branch, BranchType::Local)) = branch {
            let Some(name) = branch.name()? else {
                continue;
            };
            if name == "main" {
                continue;
            }
            if branch.is_head() {
                continue;
            }

            let branch_id = ctx
                .repo
                .find_branch(name, BranchType::Local)?
                .into_reference()
                .peel_to_commit()?
                .id();

            let fork_id = ctx.repo.merge_base(main_id, branch_id)?;

            let branch_tree_id = ctx.repo.find_commit(branch_id)?.tree_id();
            let fork_tree_id = ctx.repo.find_commit(fork_id)?.tree_id();

            if branch_tree_id == fork_tree_id {
                branches_to_delete.push(name.into());
            }
        }
    }

    if branches_to_delete.is_empty() {
        return Ok(());
    }

    let delete_args = DeleteArgs {
        names: branches_to_delete,
    };

    delete_command(ctx, &delete_args)?;

    eprintln!("Deleted: {}", delete_args.names.join(", "));

    Ok(())
}
