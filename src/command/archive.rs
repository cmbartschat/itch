use crate::{
    cli::DeleteArgs,
    command::delete::delete_command,
    ctx::Ctx,
    error::{Attempt, inner_fail},
    remote::push_tag,
    reset::pop_and_reset,
    save::save_temp,
};

pub fn archive_command(ctx: &Ctx, args: &DeleteArgs) -> Attempt {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| inner_fail("Unable to get current timestamp"))?
        .as_secs()
        .to_string();
    save_temp(ctx, "Save before archive".to_string())?;
    for branch_name in &args.names {
        let branch = ctx.repo.find_branch(branch_name, git2::BranchType::Local)?;
        let tag_name = format!("archive-{now}-{branch_name}");
        ctx.repo.tag_lightweight(
            &tag_name,
            branch.into_reference().peel_to_commit()?.as_object(),
            false,
        )?;

        push_tag(ctx, tag_name.as_str())?;
    }

    delete_command(ctx, args)?;

    pop_and_reset(ctx)?;

    Ok(())
}
