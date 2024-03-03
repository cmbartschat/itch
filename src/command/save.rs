use git2::{Error, IndexAddOption};

use crate::{cli::SaveArgs, ctx::Ctx, remote::push_branch, reset::reset_repo};

pub fn _save_command(ctx: &Ctx, args: &SaveArgs, silent: bool) -> Result<(), Error> {
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
            println!("Nothing to commit.");
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

pub fn save_command(ctx: &Ctx, args: &SaveArgs, silent: bool) -> Result<(), Error> {
    _save_command(ctx, args, silent)?;
    let branch_name = ctx
        .repo
        .branches(Some(git2::BranchType::Local))?
        .find(|c| c.as_ref().is_ok_and(|b| b.0.is_head()));

    push_branch(
        ctx,
        branch_name.unwrap().unwrap().0.name().unwrap().unwrap(),
    )?;
    reset_repo(&ctx)?;
    Ok(())
}
