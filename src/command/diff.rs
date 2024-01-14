use git2::{Error, IntoCString};

use crate::{
    cli::DiffArgs,
    ctx::Ctx,
    diff::{collapse_renames, good_diff_options},
};

#[derive(Debug)]
enum DiffPoint {
    Current,
    Ref(String),
}

#[derive(Debug)]
enum DiffIntent {
    FromFork,
    FromSaved,
    OfBranch(String),
    Range(DiffPoint, DiffPoint),
}

/*

- from fork
itch diff                -> diff fork to current
itch diff branchname     -> diff fork to branchname

itch diff (of my branch)
itch diff of branchname

itch diff unsaved


itch diff
   -> unsaved
   -> of
   -> from/to

itch diff from commitid to branchname -> diff commitid to branchname
itch diff from commitid               -> diff commitid to current
itch diff to branchname               -> diff current to branchname
itch diff from branchname             -> diff branchname to current

itch diff unsaved                     -> diff from last saved to current
itch diff from main                   -> diff
itch diff from unsaved
itch diff main

*/

fn parse_intent(parts: &Vec<String>) -> Result<DiffIntent, Error> {
    if parts.len() == 0 {
        return Ok(DiffIntent::FromFork);
    }
    if parts.len() == 1 {
        if parts[0] == "unsaved" {
            return Ok(DiffIntent::FromSaved);
        } else {
            return Err(Error::from_str("Unexpected arguments to diff."));
        }
    }

    if parts.len() == 2 {
        match parts[0].as_ref() {
            "of" => {
                return Ok(DiffIntent::OfBranch(parts[1].clone()));
            }
            "from" => {
                return Ok(DiffIntent::Range(
                    DiffPoint::Ref(parts[1].clone()),
                    DiffPoint::Current,
                ));
            }
            "to" => {
                return Ok(DiffIntent::Range(
                    DiffPoint::Current,
                    DiffPoint::Ref(parts[1].clone()),
                ));
            }
            _ => {
                return Err(Error::from_str("Expected 'of', 'from', or 'to'."));
            }
        };
    };

    if parts.len() == 4 {
        if parts[0] != "from" || parts[2] != "to" {
            return Err(Error::from_str("Expected 'from x to y' format."));
        }

        return Ok(DiffIntent::Range(
            DiffPoint::Ref(parts[1].clone()),
            DiffPoint::Ref(parts[3].clone()),
        ));
    };

    return Err(Error::from_str("Unexpected argument format."));
}

pub fn diff_command(ctx: &Ctx, args: &DiffArgs) -> Result<(), Error> {
    let base_branch = ctx.repo.find_branch("main", git2::BranchType::Local)?;
    let base_commit = base_branch.into_reference().peel_to_commit()?;

    let mut options = good_diff_options();

    let diff_options = Some(&mut options);

    let intent = parse_intent(&args.args)?;

    let mut diff = match intent {
        DiffIntent::FromFork => {
            let head_id = ctx.repo.head()?.peel_to_commit()?;

            let fork_point = ctx
                .repo
                .find_commit(ctx.repo.merge_base(base_commit.id(), head_id.id())?)?;

            ctx.repo
                .diff_tree_to_workdir(Some(&fork_point.tree()?), diff_options)?
        }
        DiffIntent::FromSaved => {
            let from_tree = ctx.repo.head()?.peel_to_tree()?;

            ctx.repo
                .diff_tree_to_workdir(Some(&from_tree), diff_options)?
        }
        DiffIntent::OfBranch(branch) => {
            let target_id = ctx
                .repo
                .find_branch(&branch, git2::BranchType::Local)?
                .into_reference()
                .peel_to_commit()?;

            let fork_point = ctx
                .repo
                .find_commit(ctx.repo.merge_base(base_commit.id(), target_id.id())?)?;

            ctx.repo.diff_tree_to_tree(
                Some(&fork_point.tree()?),
                Some(&target_id.tree()?),
                diff_options,
            )?
        }
        DiffIntent::Range(from_point, to_point) => match (&from_point, &to_point) {
            (DiffPoint::Ref(from), DiffPoint::Ref(to)) => {
                let from_tree = ctx.repo.revparse_single(&from)?.peel_to_tree()?;
                let to_tree = ctx.repo.revparse_single(&to)?.peel_to_tree()?;

                ctx.repo
                    .diff_tree_to_tree(Some(&from_tree), Some(&to_tree), diff_options)?
            }
            (DiffPoint::Current, DiffPoint::Current) => {
                return Err(Error::from_str("Cannot diff current to current."));
            }
            (DiffPoint::Current, DiffPoint::Ref(_)) => {
                return Err(Error::from_str("Cannot diff in this direction."))
            }
            (DiffPoint::Ref(from), DiffPoint::Current) => {
                let from_tree = ctx.repo.revparse_single(&from)?.peel_to_tree()?;

                ctx.repo
                    .diff_tree_to_workdir(Some(&from_tree), diff_options)?
            }
        },
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

    Ok(())
}
