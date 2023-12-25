use git2::{Error, IntoCString};

use crate::{
    cli::DiffArgs,
    ctx::Ctx,
    diff::{collapse_renames, good_diff_options},
};

pub fn diff_command(ctx: &Ctx, args: &DiffArgs) -> Result<(), Error> {
    let base_branch = ctx.repo.find_branch("main", git2::BranchType::Local)?;
    let base_commit = base_branch.into_reference().peel_to_commit()?;

    let mut options = good_diff_options();

    let diff_options = Some(&mut options);

    let mut diff = match &args.name {
        Some(branch) => {
            let target_id = ctx
                .repo
                .find_branch(&branch, git2::BranchType::Local)?
                .into_reference()
                .peel_to_commit()?;

            let fork_point = ctx
                .repo
                .find_commit(ctx.repo.merge_base(base_commit.id(), target_id.id())?)?;

            ctx.repo.diff_tree_to_tree(
                Some(&fork_point.tree()?),
                Some(&target_id.tree()?),
                diff_options,
            )?
        }
        _ => {
            let head_id = ctx.repo.head()?.peel_to_commit()?;

            let fork_point = ctx
                .repo
                .find_commit(ctx.repo.merge_base(base_commit.id(), head_id.id())?)?;

            ctx.repo
                .diff_tree_to_workdir(Some(&fork_point.tree()?), diff_options)?
        }
    };

    collapse_renames(&mut diff)?;

    diff.print(git2::DiffFormat::Patch, |_, _, line| {
        let origin = line.origin();
        let color_code = match origin {
            '+' => "\x1b[32m+",
            '-' => "\x1b[31m-",
            ' ' => " ",
            _ => "",
        };

        print!(
            "{}{}\x1b[0m",
            color_code,
            line.content().into_c_string().unwrap().to_str().unwrap()
        );
        return true;
    })?;

    Ok(())
}
