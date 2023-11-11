use git2::{Error, Repository};
use log::debug;

use crate::{cli::SyncArgs, ctx::Ctx};

fn sync_branch(repo: &mut Repository, branch: &str, base: &str) -> Result<(), Error> {
    let target_branch = repo.find_branch(&branch, git2::BranchType::Local)?;
    let start_commit = target_branch.into_reference().peel_to_commit()?;
    let base_branch = repo.find_branch(&base, git2::BranchType::Local)?;
    let base_commit = base_branch.into_reference().peel_to_commit()?;

    let main_branch = repo.find_branch("main", git2::BranchType::Local)?;
    let main_commit = main_branch.into_reference().peel_to_commit()?;
    let branch_id = repo.find_annotated_commit(base_commit.id())?;
    let onto = repo.find_annotated_commit(start_commit.id())?;
    let upstream = repo.find_annotated_commit(main_commit.id())?;

    // let fork_point = repo.merge_base(start_commit.id(), base_commit.id())?;
    let mut rebase = repo.rebase(Some(&branch_id), Some(&upstream), Some(&onto), None)?;
    let result = rebase.commit(None, &repo.signature()?, None)?;
    debug!("rebased and got {:?}", result);
    // repo.branch(&branch, result, true)?;

    Ok(())
}

pub fn sync_command(ctx: &mut Ctx, args: &SyncArgs) -> Result<(), ()> {
    for branch in &args.names {
        sync_branch(&mut ctx.repo, &branch, "main").map_err(|err| {
            debug!("Failed to sync {} due to {:?}", branch, err);
            return ();
        })?;
    }

    Ok(())
}
