use crate::{
    branch::choose_random_branch_name, cli::SplitArgs, ctx::Ctx, error::Attempt,
    reset::pop_and_reset, save::save_temp,
};

pub fn split_command(ctx: &Ctx, args: &SplitArgs) -> Attempt {
    save_temp(ctx, "Save before split".to_string())?;

    let name: String = match &args.name {
        Some(n) => {
            if n.is_empty() {
                choose_random_branch_name(ctx)
            } else {
                Ok(n.to_string())
            }
        }
        None => choose_random_branch_name(ctx),
    }?;

    let head_commit = ctx.repo.head()?.peel_to_commit()?;

    let message = format!("Splitting to {name}");

    let new_reference = ctx.repo.reference(
        name.as_str(),
        head_commit.id(),
        gix::refs::transaction::PreviousValue::MustNotExist,
        message.clone(),
    )?;

    ctx.repo.reference(
        "HEAD",
        new_reference.id(),
        gix::refs::transaction::PreviousValue::MustNotExist,
        message,
    )?;

    pop_and_reset(ctx)?;

    if ctx.can_prompt() {
        eprintln!("Split to {name}");
    }

    Ok(())
}
