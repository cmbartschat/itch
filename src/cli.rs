use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    New(NewArgs),
    Save,
    Merge,
    Delete(DeleteArgs),
    Sync,
}

#[derive(Args)]
pub struct NewArgs {
    pub name: Option<String>,
    #[arg(short, long)]
    pub base: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    pub names: Vec<String>,
}
