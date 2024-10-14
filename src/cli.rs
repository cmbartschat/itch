use clap::{Args, Parser, Subcommand};
use serde::Deserialize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short = 'v', long = "verbose", global = true, help = "Show debug logs")]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize an empty repo in the current folder")]
    Init,

    #[command(about = "Connect to a remote git service")]
    Connect(ConnectArgs),

    #[command(about = "Remove remote git service")]
    Disconnect,

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

    #[command(about = "Split the current branch into a separate workstream and switch to it")]
    Split(SplitArgs),

    #[command(about = "Show interactive UI")]
    Ui,

    #[command(about = "Clear out all save commits without reverting changes")]
    Unsave(UnsaveArgs),

    #[command(about = "Undo changes to files since the last merge")]
    Revert(RevertArgs),
}

#[derive(Args, Deserialize, Debug)]
pub struct NewArgs {
    pub name: Option<String>,
}

#[derive(Args, Deserialize, Debug)]
pub struct SplitArgs {
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

#[derive(Args, Deserialize, Debug)]
pub struct LoadArgs {
    pub name: String,
}

#[derive(Args, Deserialize, Debug)]
pub struct StatusArgs {
    pub name: Option<String>,
}

#[derive(Args, Deserialize, Debug)]
pub struct DiffArgs {
    pub args: Vec<String>,
}

#[derive(Args, Deserialize, Debug)]
pub struct ConnectArgs {
    pub url: String,
}

#[derive(Args, Deserialize, Debug)]
pub struct RevertArgs {
    pub args: Vec<String>,
}

#[derive(Args, Deserialize, Debug)]
pub struct UnsaveArgs {
    pub args: Vec<String>,
}
