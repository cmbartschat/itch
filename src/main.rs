use clap::Parser;
use cli::{Cli, Commands};
use ctx::init_ctx;
use delete_command::delete_command;
use log::LevelFilter;
use new_command::new_command;

mod base;
mod branch;
mod cli;
mod ctx;
mod delete_command;
mod new_command;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let ctx = init_ctx().expect("Could not init ctx");

    let cli = Cli::parse();

    match &cli.command {
        Commands::New(args) => new_command(&ctx, &args),
        Commands::Delete(args) => delete_command(&ctx, &args),
        _ => panic!("Not implemented."),
    }
    .expect("Failed to run command.");
}
