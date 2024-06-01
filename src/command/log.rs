use crate::{
    ctx::Ctx,
    error::{fail, inner_fail, Attempt},
    output::OutputTarget,
};
use std::fmt::Write;

pub fn log_command(ctx: &Ctx) -> Attempt {
    let mut output = OutputTarget::new()?;

    let mut repo_head = Some(ctx.repo.head()?.peel_to_commit()?);
    let mut iterations = 0;
    while let Some(current_commit) = repo_head {
        let message = &current_commit.message().unwrap_or("<invalid message>");

        let truncated_message = match message.find("\n") {
            Some(i) => &message[0..i],
            None => &message[0..],
        };

        writeln!(
            output,
            "[{}] {}",
            &current_commit.id().to_string()[0..8],
            truncated_message,
        )
        .map_err(|_| inner_fail("Failed to output data"))?;

        repo_head = current_commit.parents().next();
        iterations += 1;
        if iterations > 1000 {
            return fail("Reached limit of 1000 commits printed.");
        }
    }

    output.finish();

    Ok(())
}
