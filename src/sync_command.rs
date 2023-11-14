use git2::{Error, RebaseOptions, Repository};
use log::debug;

use crate::{cli::SyncArgs, ctx::Ctx};

fn sync_branch(repo: &mut Repository, branch: &str) -> Result<(), Error> {
    let branch_ref = repo
        .find_branch(&branch, git2::BranchType::Local)?
        .into_reference();
    let main_ref = repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference();

    let upstream_id = repo.reference_to_annotated_commit(&&main_ref)?;
    let branch_id = repo.reference_to_annotated_commit(&branch_ref)?;

    // let branch_commit = branch_ref.peel_to_commit()?;
    // let main_commit = main_ref.peel_to_commit()?;

    // let fork_point = repo.merge_base(start_commit.id(), base_commit.id())?;

    debug!("Attempting to rebase...");

    // debug!("main_commit: {:?}", main_commit.id(),);

    // debug!("branch_commit: {:?}", branch_commit.id(),);
    debug!(
        "branch: {:?}, ref: {:?}",
        branch_id.id(),
        branch_id.refname().unwrap_or("<unset>")
    );

    debug!(
        "upstream: {:?}, ref: {:?}",
        upstream_id.id(),
        upstream_id.refname().unwrap_or("<unset>")
    );
    // debug!("onto: {:?}", onto.id());

    // if branch_id.id() == upstream.id() {
    //     debug!("No change.");
    //     return Ok(());
    // }

    let mut rebase = repo.rebase(
        Some(&branch_id),
        Some(&upstream_id),
        None,
        Some(&mut RebaseOptions::new().inmemory(true)),
    )?;

    debug!("Rebase started.");

    while let Some(Ok(operation)) = rebase.next() {
        debug!("Looking at conflict: {:?}", operation);
    }

    let result = rebase.finish(Some(&repo.signature()?))?;

    // let result = rebase.commit(None, &repo.signature()?, None)?;
    debug!("rebased and got {:?}", result);
    // repo.branch(&branch, result, true)?;

    Ok(())
}

pub fn sync_command(ctx: &mut Ctx, args: &SyncArgs) -> Result<(), ()> {
    for branch in &args.names {
        sync_branch(&mut ctx.repo, &branch).map_err(|err| {
            debug!("Failed to sync {} due to {:?}", branch, err);
            return ();
        })?;
    }

    Ok(())
}
