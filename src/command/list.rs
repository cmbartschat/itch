use git2::Error;

use crate::ctx::Ctx;

pub fn list_command(ctx: &Ctx) -> Result<(), Error> {
    let (selected_color, clear_color) = if ctx.color_enabled() {
        ("\x1b[1;34m", "\x1b[0m")
    } else {
        ("", "")
    };
    let (selected_prefix, normal_prefix) = if ctx.is_pipe() {
        ("", "")
    } else {
        ("> ", "  ")
    };
    for branch in ctx.repo.branches(Some(git2::BranchType::Local))? {
        match branch {
            Ok(b) => match b.0.name() {
                Ok(Some(name)) => {
                    if b.0.is_head() {
                        println!("{selected_color}{selected_prefix}{name}{clear_color}");
                    } else {
                        println!("{normal_prefix}{name}");
                    }
                }
                _ => println!("<Invalid branch>"),
            },
            _ => println!("<Invalid branch>"),
        }
    }

    Ok(())
}
