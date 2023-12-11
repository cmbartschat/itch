use git2::Error;

use crate::ctx::Ctx;

pub fn list_command(ctx: &Ctx) -> Result<(), Error> {
    for branch in ctx.repo.branches(Some(git2::BranchType::Local))? {
        match branch {
            Ok(b) => match b.0.name() {
                Ok(Some(name)) => println!("{} {}", if b.0.is_head() { "*" } else { " " }, name),
                _ => println!("<Invalid branch>"),
            },
            _ => println!("<Invalid branch>"),
        }
    }

    Ok(())
}
