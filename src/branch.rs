use git2::{BranchType, ErrorCode};
use log::debug;

use crate::ctx::Ctx;

pub fn local_branch_exists(ctx: &Ctx, branch: &str) -> Result<bool, ()> {
    match ctx.repo.find_branch(branch, BranchType::Local) {
        Ok(_) => Ok(true),
        Err(err) => {
            if err.code() == ErrorCode::NotFound {
                return Ok(false);
            }
            debug!("Error resolving branch with code: {:?}", err.code());
            Err(())
        }
    }
}

pub fn choose_random_branch_name(ctx: &Ctx) -> Result<String, ()> {
    for i in 1..100 {
        let new_name = format!("b{}", i);
        let exists = local_branch_exists(ctx, &new_name)?;
        if !exists {
            return Ok(new_name);
        }
    }
    return Err(());
}
