use git2::{Error, ResetType};

use crate::ctx::Ctx;

pub fn reset_repo(ctx: &Ctx) -> Result<(), Error> {
    let object = ctx.repo.head()?.peel_to_commit()?.into_object();
    ctx.repo.reset(&object, ResetType::Mixed, None)?;
    Ok(())
}
