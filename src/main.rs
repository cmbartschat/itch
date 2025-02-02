#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::redundant_else)]

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
mod editor;
mod error;
mod output;
mod path;
mod print;
mod prompt;
mod remote;
mod reset;
mod save;
mod sync;
mod timer;

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init();
    }

    let res = run_command(&cli);

    if let Err(e) = res {
        eprintln!("Failed with error: {}", e.message());
    }
}
