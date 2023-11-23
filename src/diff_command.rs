use git2::IntoCString;
use log::debug;

use crate::{base::resolve_base, ctx::Ctx};

pub fn diff_command(ctx: &Ctx) -> Result<(), ()> {
    let base = resolve_base(&None).map_err(|_| ())?;
    let base_branch = ctx
        .repo
        .find_branch(&base, git2::BranchType::Local)
        .map_err(|_| ())?;
    let base_tree = base_branch
        .into_reference()
        .peel_to_tree()
        .map_err(|_| ())?;
    debug!("{:?}", base_tree);
    let diff = ctx
        .repo
        .diff_tree_to_workdir(Some(&base_tree), None)
        .map_err(|_| ())?;
    diff.print(git2::DiffFormat::Patch, |_, _, line| {
        print!(
            "{}{}",
            line.origin(),
            line.content().into_c_string().unwrap().to_str().unwrap()
        );
        return true;
    })
    .map_err(|_| ())?;
    Ok(())
}
