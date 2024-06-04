use git2::IndexAddOption;
use macros::{timer_done, timer_next, timer_start};

use crate::{cli::SaveArgs, consts::TEMP_COMMIT_PREFIX, ctx::Ctx, error::Attempt};

pub fn save(ctx: &Ctx, args: &SaveArgs, silent: bool) -> Attempt {
    timer_start!("save");

    let repo = &ctx.repo;

    let mut index = repo.index()?;

    timer_next!("before index");
    index.add_all(["*"], IndexAddOption::all(), None)?;
    timer_next!("after index");
    let index_commit = index.write_tree()?;

    timer_next!("after write");
    let tree = repo.find_tree(index_commit)?;
    timer_next!("after find");

    let mut message = args.message.join(" ");
    if message.len() == 0 {
        message = String::from("Save");
    }

    let signature = repo.signature()?;

    let parent = repo.head()?.peel_to_commit()?;

    timer_next!("after peel");

    if index_commit == parent.tree_id() {
        if !silent {
            eprintln!("Nothing to commit.");
        }
        timer_done!();
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

    timer_next!("after commit");

    timer_done!();

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
