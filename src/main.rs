use clap::Parser;
use cli::Cli;
use command::run_command;
use log::LevelFilter;

mod branch;
mod cli;
mod command;
mod consts;
mod ctx;
mod diff;
mod output;
mod path;
mod reset;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init();
    }

    let res = run_command(&cli);

    if let Err(e) = res.await {
        eprintln!("Failed with error: {}", e.message());
    }
}
