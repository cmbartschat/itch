use git2::{BranchType, Error};

use crate::{cli::DeleteArgs, ctx::Ctx, delete_command::delete_command};

pub fn prune_command(ctx: &Ctx) -> Result<(), Error> {
    let mut branches_to_delete: Vec<String> = vec![];

    let main_ref = ctx
        .repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference();

    for branch in ctx.repo.branches(Some(git2::BranchType::Local))? {
        match branch {
            Ok((branch, BranchType::Local)) => {
                let Some(name) = branch.name()?.clone() else {
                    continue;
                };
                if name == "main" {
                    continue;
                }
                if branch.is_head() {
                    continue;
                }

                let branch_id = ctx.repo.reference_to_annotated_commit(
                    &ctx.repo
                        .find_branch(name, BranchType::Local)?
                        .into_reference(),
                )?;

                let analysis = ctx.repo.merge_analysis_for_ref(&main_ref, &[&branch_id])?.0;

                if analysis.is_up_to_date() {
                    branches_to_delete.push(name.into());
                }
            }
            _ => {}
        }
    }

    if branches_to_delete.is_empty() {
        return Ok(());
    }

    let delete_args = DeleteArgs {
        names: branches_to_delete,
    };

    delete_command(&ctx, &delete_args)?;

    println!("Deleted: {}", delete_args.names.join(", "));

    Ok(())
}
