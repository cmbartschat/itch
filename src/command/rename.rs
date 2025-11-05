use anyhow::bail;

use crate::{cli::RenameArgs, ctx::Ctx, error::Attempt};

pub fn rename_command(ctx: &Ctx, args: &RenameArgs) -> Attempt {
    let Some(current) = ctx.repo.head_ref()? else {
        bail!("No current branch is active.");
    };
    let old_name = current.name().shorten().to_string();
    if old_name == "main" {
        bail!("Cannot rename the main branch");
    }
    let message = format!("Renaming current branch to {}", args.name);

    let new_reference = ctx.repo.reference(
        args.name.as_str(),
        current.id(),
        gix::refs::transaction::PreviousValue::MustNotExist,
        message.clone(),
    )?;

    ctx.repo.reference(
        "HEAD",
        new_reference.id(),
        gix::refs::transaction::PreviousValue::MustNotExist,
        message,
    )?;
    todo!();
    // try_delete_remote_branch(ctx, &old_name);
    if ctx.can_prompt() {
        eprintln!("Renamed to {}", args.name);
    }
    Ok(())
}
