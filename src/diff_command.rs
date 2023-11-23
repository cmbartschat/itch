use git2::{Error, IntoCString};
use log::debug;

use crate::{base::resolve_base, cli::DiffArgs, ctx::Ctx};

fn _diff_command(ctx: &Ctx, args: &DiffArgs) -> Result<(), Error> {
    let base = resolve_base(&None).map_err(|_| Error::from_str("Unable to resolve base."))?;
    let base_branch = ctx.repo.find_branch(&base, git2::BranchType::Local)?;
    let base_tree = base_branch.into_reference().peel_to_tree()?;
    debug!("{:?}", base_tree);
    let diff = match &args.target {
        Some(branch) => {
            let target_branch = ctx.repo.find_branch(&branch, git2::BranchType::Local)?;
            let target_tree = target_branch.into_reference().peel_to_tree()?;
            ctx.repo
                .diff_tree_to_tree(Some(&base_tree), Some(&target_tree), None)?
        }
        _ => ctx.repo.diff_tree_to_workdir(Some(&base_tree), None)?,
    };
    diff.print(git2::DiffFormat::Patch, |_, _, line| {
        print!(
            "{}{}",
            line.origin(),
            line.content().into_c_string().unwrap().to_str().unwrap()
        );
        return true;
    })?;
    Ok(())
}

pub fn diff_command(ctx: &Ctx, args: &DiffArgs) -> Result<(), ()> {
    _diff_command(&ctx, &args).map_err(|e| {
        println!("Failed to diff: {}", e.message());
    })
}
