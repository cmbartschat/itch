use clap::Parser;
use cli::{Cli, Commands};
use ctx::init_ctx;
use delete_command::delete_command;
use diff_command::diff_command;
use list_command::list_command;
use load_command::load_command;
use log::LevelFilter;
use log_command::log_command;
use merge_command::merge_command;
use new_command::new_command;
use save_command::save_command;
use squash_command::squash_command;
use status_command::status_command;
use sync_command::sync_command;

mod branch;
mod cli;
mod ctx;
mod delete_command;
mod diff_command;
mod list_command;
mod load_command;
mod log_command;
mod merge_command;
mod new_command;
mod reset;
mod save_command;
mod squash_command;
mod status_command;
mod sync_command;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let ctx = init_ctx().expect("Could not init ctx");

    let cli = Cli::parse();

    let res = match &cli.command {
        Commands::Delete(args) => delete_command(&ctx, &args),
        Commands::Diff(args) => diff_command(&ctx, &args),
        Commands::List => list_command(&ctx),
        Commands::Load(args) => load_command(&ctx, &args),
        Commands::Log => log_command(&ctx),
        Commands::Merge => merge_command(&ctx),
        Commands::New(args) => new_command(&ctx, &args),
        Commands::Save(args) => save_command(&ctx, &args),
        Commands::Status(args) => status_command(&ctx, &args),
        Commands::Squash => squash_command(&ctx),
        Commands::Sync(args) => sync_command(&ctx, &args),
    };

    if let Err(e) = res {
        eprintln!("Failed with error: {}", e.message());
    }
}
