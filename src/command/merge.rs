use anyhow::bail;
use gix::ObjectId;

use crate::{
    branch::find_main,
    ctx::Ctx,
    error::{Attempt, Maybe},
    remote::{try_pull_main, try_push_main},
};

fn combine_branches(ctx: &Ctx) -> Maybe<ObjectId> {
    let repo = &ctx.repo;

    let main_ref = find_main(ctx)?;

    // let branch_id = repo.reference_to_annotated_commit(&repo.head()?)?;

    todo!();
    // let analysis = ctx.repo.merge_analysis_for_ref(&main_ref, &[&branch_id])?.0;

    // if analysis.is_fast_forward() {
    //     return Ok(branch_id.id());
    // }

    bail!("Must be synced on main")
}

pub fn merge_command(ctx: &Ctx) -> Attempt {
    let head = ctx.repo.head()?;
    let head_name = head.name().shorten().to_string();

    if head_name == "refs/heads/main" {
        bail!("Cannot merge from main.");
    }

    try_pull_main(ctx);

    let resolved_commit = combine_branches(ctx)?;

    let reflog_message = format!("Merged from {head_name}");

    find_main(ctx)?.set_target_id(resolved_commit, reflog_message)?;

    try_push_main(ctx);

    Ok(())
}
