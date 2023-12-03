use crate::ctx::Ctx;

pub fn list_command(ctx: &Ctx) -> Result<(), ()> {
    for branch in ctx
        .repo
        .branches(Some(git2::BranchType::Local))
        .map_err(|e| {
            println!("Failed to list branches: {}", e.to_string());
        })?
    {
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
