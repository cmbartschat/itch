use crate::{ctx::Ctx, error::Attempt, remote::disconnect_remote};

pub fn disconnect_command(ctx: &Ctx) -> Attempt {
    disconnect_remote(ctx)?;
    Ok(())
}
