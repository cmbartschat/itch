use std::io::IsTerminal;
use std::{env, io::stdout};

use crate::error::Attempt;
use crate::{
    cli::{Cli, Commands},
    ctx::{init_ctx, Mode},
};

use self::{
    delete::delete_command, diff::diff_command, list::list_command, load::load_command,
    log::log_command, merge::merge_command, new::new_command, prune::prune_command,
    save::save_command, squash::squash_command, status::status_command, sync::sync_command,
    ui::ui_command, unsave::unsave_command,
};

mod delete;
mod diff;
mod list;
mod load;
mod log;
mod merge;
mod new;
mod prune;
mod save;
mod squash;
mod status;
mod sync;
mod ui;
mod unsave;

pub async fn run_command(cli: &Cli) -> Attempt {
    let mut ctx = init_ctx()?;
    ctx.set_mode(if stdout().lock().is_terminal() {
        Mode::Cli
    } else {
        Mode::Pipe
    });
    if env::var_os("NO_COLOR").is_some() {
        ctx.disable_color();
    }

    match &cli.command {
        Commands::Prune => prune_command(&ctx),
        Commands::Delete(args) => delete_command(&ctx, &args),
        Commands::Diff(args) => diff_command(&ctx, &args),
        Commands::List => list_command(&ctx),
        Commands::Load(args) => load_command(&ctx, &args),
        Commands::Log => log_command(&ctx),
        Commands::Merge => merge_command(&ctx),
        Commands::New(args) => new_command(&ctx, &args),
        Commands::Save(args) => save_command(&ctx, &args, false),
        Commands::Status(args) => status_command(&ctx, &args),
        Commands::Squash => squash_command(&ctx),
        Commands::Sync(args) => sync_command(&ctx, &args),
        Commands::Ui => ui_command(&ctx).await,
        Commands::Unsave => unsave_command(&ctx),
    }
}
