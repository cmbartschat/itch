use git2::{BranchType, Error, ErrorCode};

use crate::ctx::Ctx;

pub fn local_branch_exists(ctx: &Ctx, branch: &str) -> Result<bool, Error> {
    match ctx.repo.find_branch(branch, BranchType::Local) {
        Ok(_) => Ok(true),
        Err(err) => {
            if err.code() == ErrorCode::NotFound {
                return Ok(false);
            }
            Err(err)
        }
    }
}

pub fn choose_random_branch_name(ctx: &Ctx) -> Result<String, Error> {
    for i in 1..100 {
        let new_name = format!("b{}", i);
        let exists = local_branch_exists(ctx, &new_name)?;
        if !exists {
            return Ok(new_name);
        }
    }
    return Err(Error::from_str("Could not autogenerate branch name."));
}

pub fn get_head_name(ctx: &Ctx) -> Result<String, Error> {
    let repo_head = ctx.repo.head()?;
    let head_name_str = repo_head.name().unwrap();
    Ok(head_name_str[head_name_str.rfind("/").map_or(0, |e| e + 1)..].to_owned())
}
