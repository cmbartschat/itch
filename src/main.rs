#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::redundant_else)]

use clap::Parser;
use cli::Cli;
use command::run_command;
use log::LevelFilter;

use crate::error::Attempt;

mod branch;
mod cli;
mod command;
mod commit;
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

fn main() -> Attempt {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init();
    }

    run_command(&cli)
}
