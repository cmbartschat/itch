use crate::{ctx::Ctx, error::Attempt, remote::disconnect_remote};

pub fn disconnect_command(ctx: &Ctx) -> Attempt {
    println!("you want me to disconnect.");
    disconnect_remote(ctx)?;
    Ok(())
}
