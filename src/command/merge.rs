use git2::{Error, Oid};

use crate::{ctx::Ctx, remote::push_main};

fn combine_branches(ctx: &Ctx) -> Result<Oid, Error> {
    let repo = &ctx.repo;

    let main_ref = repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference();

    let branch_id = repo.reference_to_annotated_commit(&repo.head()?)?;

    let analysis = ctx.repo.merge_analysis_for_ref(&main_ref, &[&branch_id])?.0;

    if analysis.is_fast_forward() {
        return Ok(branch_id.id());
    }

    return Err(Error::from_str("Must be synced on main"));
}

pub fn merge_command(ctx: &Ctx) -> Result<(), Error> {
    let head = ctx.repo.head()?;
    let head_name = head.name().expect("No valid head name.");

    if head_name == "refs/heads/main" {
        return Err(Error::from_str("Cannot merge from main."));
    }

    let resolved_commit = combine_branches(ctx)?;

    let reflog_message = format!("Merged from {head_name}");

    ctx.repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference()
        .set_target(resolved_commit, &reflog_message)?;

    match push_main(ctx) {
        Err(e) => println!("Skipping remote push due to: {}", e.message()),
        _ => {}
    }

    Ok(())
}
