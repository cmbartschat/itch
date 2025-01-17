use git2::IndexAddOption;

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

    let mut index = repo.index()?;
    index.add_all(["*"], IndexAddOption::all(), None)?;
    let index_commit = index.write_tree()?;

    let tree = repo.find_tree(index_commit)?;

    let message = resolve_commit_message(&args.message).unwrap_or_else(|| "Save".into());

    let signature = repo.signature()?;

    let parent = repo.head()?.peel_to_commit()?;

    if index_commit == parent.tree_id() {
        if !silent {
            eprintln!("Nothing to commit.");
        }
        return Ok(());
    }

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &[&parent],
    )?;

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
