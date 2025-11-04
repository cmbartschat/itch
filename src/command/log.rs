use crate::{ctx::Ctx, error::Attempt, output::OutputTarget};
use anyhow::{anyhow, bail};
use std::fmt::Write;

pub fn log_command(ctx: &Ctx) -> Attempt {
    let mut output = OutputTarget::new();

    let mut repo_head = Some(ctx.repo.head()?.peel_to_commit()?);
    let mut iterations = 0;
    while let Some(current_commit) = repo_head {
        let summary = current_commit.message()?.summary();

        writeln!(
            output,
            "[{}] {}",
            &current_commit.id().to_string()[0..8],
            summary,
        )
        .map_err(|_| anyhow!("Failed to output data"))?;

        match current_commit
            .parent_ids()
            .next()
            .map(|i| ctx.repo.find_commit(i))
        {
            Some(i) => {
                repo_head = Some(i?);
            }
            None => {
                repo_head = None;
            }
        };
        iterations += 1;
        if iterations > 1000 {
            bail!("Reached limit of 1000 commits printed.");
        }
    }

    output.finish();

    Ok(())
}
