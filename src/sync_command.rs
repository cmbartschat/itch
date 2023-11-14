use git2::{Error, Oid, RebaseOptions, Repository};
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

    debug!("Attempting to rebase...");

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

    let mut rebase = repo.rebase(
        Some(&branch_id),
        Some(&upstream_id),
        None,
        Some(&mut RebaseOptions::new().inmemory(true)),
    )?;

    debug!("Rebase started.");

    let mut final_id: Option<Oid> = None;

    while let Some(Ok(operation)) = rebase.next() {
        debug!("Looking at operation: {:?}", operation);
        let id = rebase.commit(None, &repo.signature()?, None)?;
        debug!("Committed: {}", id);
        final_id = Some(id);
    }

    rebase.finish(Some(&repo.signature()?))?;

    match final_id {
        Some(id) => {
            let final_commit = repo.find_commit(id)?;
            repo.branch(&branch, &final_commit, true)?;
            debug!("rebased and updated {:?} with {:?}", branch, final_commit);
        }
        _ => {
            debug!("Nothing to update.");
        }
    }

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
