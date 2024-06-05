use git2::Oid;

use crate::{
    ctx::Ctx,
    error::{fail, Attempt, Maybe},
    remote::try_push_main,
};

fn combine_branches(ctx: &Ctx) -> Maybe<Oid> {
    let repo = &ctx.repo;

    let main_ref = repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference();

    let branch_id = repo.reference_to_annotated_commit(&repo.head()?)?;

    let analysis = ctx.repo.merge_analysis_for_ref(&main_ref, &[&branch_id])?.0;

    if analysis.is_fast_forward() {
        return Ok(branch_id.id());
    }

    fail("Must be synced on main")
}

pub fn merge_command(ctx: &Ctx) -> Attempt {
    let head = ctx.repo.head()?;
    let head_name = head.name().expect("No valid head name.");

    if head_name == "refs/heads/main" {
        return fail("Cannot merge from main.");
    }

    let resolved_commit = combine_branches(ctx)?;

    let reflog_message = format!("Merged from {head_name}");

    ctx.repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference()
        .set_target(resolved_commit, &reflog_message)?;

    try_push_main(ctx);

    Ok(())
}
