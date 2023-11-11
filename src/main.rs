use clap::Parser;
use cli::{Cli, Commands};
use ctx::init_ctx;
use delete_command::delete_command;
use list_command::list_command;
use log::LevelFilter;
use new_command::new_command;
use sync_command::sync_command;

mod base;
mod branch;
mod cli;
mod ctx;
mod delete_command;
mod list_command;
mod new_command;
mod sync_command;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let mut ctx = init_ctx().expect("Could not init ctx");

    let cli = Cli::parse();

    match &cli.command {
        Commands::New(args) => new_command(&ctx, &args),
        Commands::Delete(args) => delete_command(&ctx, &args),
        Commands::List => list_command(&ctx),
        Commands::Sync(args) => sync_command(&mut ctx, &args),
        _ => panic!("Not implemented."),
    }
    .expect("Failed to run command.");
}
