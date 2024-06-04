use clap::Parser;
use cli::Cli;
use command::run_command;
use log::LevelFilter;
use macros::{timer_next, timer_start};

mod branch;
mod cli;
mod command;
mod consts;
mod ctx;
mod diff;
mod editor;
mod error;
mod output;
mod path;
mod remote;
mod reset;
mod save;
mod sync;
mod timer;

#[tokio::main]
async fn main() {
    timer_start!("main");
    let cli = Cli::parse();
    timer_next!("parsed");

    if cli.verbose {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init();
        timer_next!("init logger");
    }

    let res = run_command(&cli);

    if let Err(e) = res.await {
        eprintln!("Failed with error: {}", e.message());
    }
    timer_next!("run command");
}
