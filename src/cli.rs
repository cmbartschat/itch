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
    New(NewArgs),
    Save(SaveArgs),
    Load(LoadArgs),
    Delete(DeleteArgs),
    Sync(SyncArgs),
    List,
    Status(StatusArgs),
    Diff(DiffArgs),
    Log,
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

#[derive(Args)]
pub struct SyncArgs {
    pub names: Vec<String>,
}

#[derive(Args)]
pub struct SaveArgs {
    pub message: Vec<String>,
}

#[derive(Args)]
pub struct LoadArgs {
    pub target: String,
}

#[derive(Args)]
pub struct StatusArgs {
    pub target: Option<String>,
}

#[derive(Args)]
pub struct DiffArgs {
    pub target: Option<String>,
}
