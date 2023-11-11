use clap::Parser;
use ctx::init_ctx;
use log::LevelFilter;
use new_command::{new_command, NewCommandArgs};

mod base;
mod branch;
mod ctx;
mod new_command;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(  version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name)
    }

    let ctx = init_ctx().expect("Could not init ctx");

    let _ = new_command(
        &ctx,
        NewCommandArgs {
            name: None,
            base: None,
        },
    );
}
