use anyhow::bail;

use crate::{
    branch::find_main, cli::DeleteArgs, command::delete::delete_command, ctx::Ctx, error::Attempt,
    reset::pop_and_reset, save::save_temp,
};

pub fn prune_command(ctx: &Ctx) -> Attempt {
    let mut branches_to_delete: Vec<String> = vec![];

    let main_id = find_main(ctx)?.peel_to_id()?;

    save_temp(ctx, "Save before prune".into())?;

    for branch in ctx.repo.references()?.local_branches()?.into_iter() {
        let Ok(mut branch) = branch else {
            bail!("Invalid branch");
        };

        let name = branch.name().shorten().to_string();

        if name == "main" {
            continue;
        };

        let branch_commit = branch.peel_to_commit()?;

        let fork_id = ctx.repo.merge_base(main_id, branch_commit.id())?;

        let branch_tree_id = branch_commit.tree_id()?;
        let fork_tree_id = ctx.repo.find_commit(fork_id)?.tree_id()?;

        if branch_tree_id == fork_tree_id {
            branches_to_delete.push(name);
        }
    }

    pop_and_reset(ctx)?;

    if branches_to_delete.is_empty() {
        return Ok(());
    }

    let delete_args = DeleteArgs {
        names: branches_to_delete,
    };

    delete_command(ctx, &delete_args)?;

    if ctx.can_prompt() {
        eprintln!("Deleted: {}", delete_args.names.join(", "));
    }

    Ok(())
}
