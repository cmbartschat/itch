use git2::{BranchType, ErrorCode};

use crate::{
    ctx::Ctx,
    error::{Maybe, fail, inner_fail},
};

pub fn local_branch_exists(ctx: &Ctx, branch: &str) -> Maybe<bool> {
    match ctx.repo.find_branch(branch, BranchType::Local) {
        Ok(_) => Ok(true),
        Err(err) => {
            if err.code() == ErrorCode::NotFound {
                return Ok(false);
            }
            Err(err.into())
        }
    }
}

pub fn choose_random_branch_name(ctx: &Ctx) -> Maybe<String> {
    for i in 1..100 {
        let new_name = format!("b{i}");
        let exists = local_branch_exists(ctx, &new_name)?;
        if !exists {
            return Ok(new_name);
        }
    }
    fail!("Could not autogenerate branch name.")
}

pub fn get_current_branch(ctx: &Ctx) -> Maybe<String> {
    for branch in ctx.repo.branches(Some(git2::BranchType::Local))? {
        let branch = branch?.0;
        if !branch.is_head() {
            continue;
        }

        let name_attempt = branch
            .name()
            .map_err(|_| inner_fail!("Invalid branch name"))?;

        if let Some(name) = name_attempt {
            return Ok(name.to_string());
        } else {
            return fail!("Current branch appears to have no name");
        }
    }

    fail!("Failed to locate active branch")
}
