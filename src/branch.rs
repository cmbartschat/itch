use anyhow::bail;
use gix::Reference;

use crate::{ctx::Ctx, error::Maybe};

pub fn local_branch_exists(ctx: &Ctx, branch: &str) -> Maybe<bool> {
    Ok(ctx.repo.try_find_reference(branch)?.is_some())
}

pub fn choose_random_branch_name(ctx: &Ctx) -> Maybe<String> {
    for i in 1..100 {
        let new_name = format!("b{i}");
        let exists = local_branch_exists(ctx, &new_name)?;
        if !exists {
            return Ok(new_name);
        }
    }
    bail!("Could not autogenerate branch name.")
}

pub fn get_current_branch(ctx: &Ctx) -> Maybe<String> {
    match ctx
        .repo
        .head_ref()?
        .map(|e| e.name().as_partial_name().to_string())
    {
        Some(e) => Ok(e),
        None => bail!("Failed to locate active branch"),
    }
}

pub fn find_branch<'a>(ctx: &'a Ctx, name: &str) -> Maybe<Reference<'a>> {
    Ok(ctx
        .repo
        .references()?
        .local_branches()?
        .map(|f| f.unwrap())
        .find(|f| f.name().shorten().to_string() == name)
        .unwrap())
}

pub fn find_main<'a>(ctx: &'a Ctx) -> Maybe<Reference<'a>> {
    find_branch(ctx, "main")
}
