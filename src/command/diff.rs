use git2::{Delta, Error, IntoCString};

use crate::{
    cli::DiffArgs,
    ctx::Ctx,
    diff::{collapse_renames, good_diff_options},
};

pub fn diff_command(ctx: &Ctx, args: &DiffArgs) -> Result<(), Error> {
    let base_branch = ctx.repo.find_branch("main", git2::BranchType::Local)?;
    let base_tree = base_branch.into_reference().peel_to_tree()?;

    let mut options = good_diff_options();

    let diff_options = Some(&mut options);

    let mut diff = match &args.target {
        Some(branch) => {
            let target_branch = ctx.repo.find_branch(&branch, git2::BranchType::Local)?;
            let target_tree = target_branch.into_reference().peel_to_tree()?;
            ctx.repo
                .diff_tree_to_tree(Some(&base_tree), Some(&target_tree), diff_options)?
        }
        _ => ctx
            .repo
            .diff_tree_to_workdir(Some(&base_tree), diff_options)?,
    };

    collapse_renames(&mut diff)?;

    diff.print(git2::DiffFormat::Patch, |_, _, line| {
        let origin = line.origin();
        let color_code = match origin {
            '+' => "\x1b[32m+",
            '-' => "\x1b[31m-",
            ' ' => " ",
            _ => "",
        };

        print!(
            "{}{}\x1b[0m",
            color_code,
            line.content().into_c_string().unwrap().to_str().unwrap()
        );
        return true;
    })?;

    diff.deltas().try_for_each(|delta| -> Result<(), Error> {
        return match delta.status() {
            Delta::Untracked => {
                println!(
                    "New file: {}",
                    delta.new_file().path().unwrap().to_str().unwrap()
                );
                let blob = ctx
                    .repo
                    .blob_path(delta.new_file().path().expect("New file has no path"))?;

                let file = ctx.repo.find_blob(blob)?;
                if file.is_binary() {
                    println!("(Binary file)");
                } else {
                    String::from_utf8_lossy(file.content())
                        .lines()
                        .for_each(|line| {
                            println!("\x1b[32m+{}\x1b[0m", line);
                        });
                }

                Ok(())
            }
            _ => Ok(()),
        };
    })?;

    Ok(())
}
