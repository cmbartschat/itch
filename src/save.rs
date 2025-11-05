use std::os::unix::fs::MetadataExt;

use anyhow::bail;
use gix::{
    bstr::ByteSlice,
    index::{
        entry::{Flags, Mode, Stat},
        fs::Metadata,
    },
    objs::Blob,
    progress::Discard,
    worktree::Index,
};

use crate::{cli::SaveArgs, consts::TEMP_COMMIT_PREFIX, ctx::Ctx, error::Attempt};

pub fn resolve_commit_message(message_parts: &[String]) -> Option<String> {
    let joined = message_parts.join(" ");
    let trimmed = joined.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

pub fn save(ctx: &Ctx, args: &SaveArgs, silent: bool) -> Attempt {
    let repo = &ctx.repo;

    let status = repo.status(Discard)?;

    let worktree_index = status.into_index_worktree_iter(vec![])?;
    let mut index = ctx.repo.index()?.as_ref().clone();

    eprintln!(
        "Initial index entries: {:?}",
        index.entries() // .iter()
                        // .map(|f| format!("{:?}: {:?}", f.stage(), f))
    );

    for worktree_item in worktree_index {
        let worktree_item = worktree_item?;
        let Some(status) = worktree_item.summary() else {
            eprintln!("No status for item: {worktree_item:?}");
            continue;
        };

        let Some(entry) = index.entry_mut_by_path_and_stage(
            worktree_item.rela_path(),
            gix::index::entry::Stage::Unconflicted,
        ) else {
            eprintln!("Entry doesn't exist, status: {status:?}");
            if status != Summary::Added {
                bail!("Missing entries should be added in the workdir");
            }
            let stat = Metadata::from_path_no_follow(worktree_item.rela_path().to_path()?)?;
            let data = std::fs::read(worktree_item.rela_path().to_os_str()?)?;
            let id = ctx.repo.write_blob(data)?.detach();
            index.dangerously_push_entry(
                Stat::from_fs(&stat)?,
                id,
                Flags::UPDATE,
                Mode::FILE,
                worktree_item.rela_path(),
            );
            index.sort_entries();
            continue;
        };

        type Summary = gix::status::index_worktree::iter::Summary;

        match status {
            Summary::Removed => entry.flags.insert(Flags::REMOVE),
            Summary::Added => todo!(),
            Summary::Modified => {
                let data = std::fs::read(worktree_item.rela_path().to_os_str()?)?;
                let blob = Blob { data: data };
                let id = ctx.repo.write_object(blob)?;
                entry.id = id.detach();
                entry.flags |= Flags::UPDATE;
            }
            Summary::TypeChange => todo!(),
            Summary::Renamed => todo!(),
            Summary::Copied => todo!(),
            Summary::IntentToAdd => todo!(),
            Summary::Conflict => todo!(),
        }
    }

    eprintln!("After worktree updates: {:?}", index.entries());
    let mut write_options = gix::index::write::Options::default();
    write_options.extensions = gix::index::write::Extensions::All;
    index.write(write_options)?;

    // gix::index::extension::decode

    // todo!();
    // index.add_all(["*"], IndexAddOption::all(), None)?;
    let Some(tree_id) = index.tree().map(|t| t.id) else {
        bail!("No tree attached to index");
    };

    let parent = repo.head()?.peel_to_commit()?;

    if tree_id == parent.tree_id()? {
        eprintln!("[debug] Tree is the same as parent");
        if !silent {
            eprintln!("Nothing to commit.");
        }
        return Ok(());
    }

    let message = resolve_commit_message(&args.message).unwrap_or_else(|| "Save".into());

    repo.commit("HEAD", &message, tree_id, vec![parent.id()])?;

    Ok(())
}

pub fn save_temp(ctx: &Ctx, message: String) -> Attempt {
    save(
        ctx,
        &SaveArgs {
            message: vec![TEMP_COMMIT_PREFIX.to_string(), message],
        },
        true,
    )
}
