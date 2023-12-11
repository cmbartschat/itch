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
    #[command(about = "Start a new branch")]
    New(NewArgs),

    #[command(about = "Save changes with a message")]
    Save(SaveArgs),

    #[command(about = "Load up the changes in another branch")]
    Load(LoadArgs),

    #[command(about = "Show the status of the current branch")]
    Status(StatusArgs),

    #[command(about = "Show the diff between the main and the current state")]
    Diff(DiffArgs),

    #[command(about = "List branches")]
    List,

    #[command(about = "Delete a branch")]
    Delete(DeleteArgs),

    #[command(about = "Show the commit history")]
    Log,

    #[command(about = "Apply current changes to the main branch")]
    Merge,

    #[command(about = "Bring the latest changes from main into this branch")]
    Sync(SyncArgs),

    #[command(about = "Flatten the current saves into one commit")]
    Squash,

    #[command(about = "Prune unneeded branches")]
    Prune,
}

#[derive(Args)]
pub struct NewArgs {
    pub name: Option<String>,
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
