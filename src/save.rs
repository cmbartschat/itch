use git2::IndexAddOption;

use crate::{cli::SaveArgs, consts::TEMP_COMMIT_PREFIX, ctx::Ctx, error::Attempt};

pub fn save(ctx: &Ctx, args: &SaveArgs, silent: bool) -> Attempt {
    let repo = &ctx.repo;

    let mut index = repo.index()?;
    index.add_all(["*"], IndexAddOption::all(), None)?;
    let index_commit = index.write_tree()?;

    let tree = repo.find_tree(index_commit)?;

    let mut message = args.message.join(" ");
    if message.len() == 0 {
        message = String::from("Save");
    }

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

pub fn save_temp(ctx: &Ctx) -> Attempt {
    save(
        ctx,
        &SaveArgs {
            message: vec![
                TEMP_COMMIT_PREFIX.to_string(),
                "Save before sync".to_owned(),
            ],
        },
        true,
    )
}
