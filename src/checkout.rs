use std::{path::PathBuf, sync::atomic::AtomicBool};

use anyhow::Ok;
use gix::{Commit, progress::Discard, worktree::stack::state::attributes::Source};

use crate::{ctx::Ctx, error::Attempt};

pub fn checkout(ctx: &Ctx, commit: Commit) -> Attempt {
    let mut options = ctx.repo.checkout_options(Source::IdMappingThenWorktree)?;
    options.overwrite_existing = true;

    let mut index_state = ctx.repo.index_from_tree(&commit.tree()?.id)?.into_parts().0;
    let dir: PathBuf = ctx.repo.path().to_path_buf();
    let objects = ctx.repo.clone().objects.into_arc()?;
    let files = Discard; // &dyn gix::features::progress::Count;
    let bytes = Discard; // &dyn gix::features::progress::Count;
    let should_interrupt: AtomicBool = Default::default();

    let outcome = gix::worktree::state::checkout(
        &mut index_state,
        dir,
        objects,
        &files,
        &bytes,
        &should_interrupt,
        options,
    )?;

    eprintln!("Checkout finished: {:?}", outcome);

    Ok(())
}
