use crate::{
    cli::DiffArgs,
    ctx::Ctx,
    diff::{collapse_renames, good_diff_options},
    error::{fail, Attempt, Maybe},
    output::OutputTarget,
};

use std::fmt::Write;

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

fn parse_intent(parts: &[String]) -> Maybe<DiffIntent> {
    if parts.is_empty() {
        return Ok(DiffIntent::FromFork);
    }
    if parts.len() == 1 {
        if parts[0] == "unsaved" {
            return Ok(DiffIntent::FromSaved);
        } else {
            return fail("Unexpected arguments to diff.");
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
                return fail("Expected 'of', 'from', or 'to'.");
            }
        };
    };

    if parts.len() != 4 {
        fail("Unexpected argument format.")
    } else if parts[0] != "from" || parts[2] != "to" {
        fail("Expected 'from x to y' format.")
    } else {
        Ok(DiffIntent::Range(
            DiffPoint::Ref(parts[1].clone()),
            DiffPoint::Ref(parts[3].clone()),
        ))
    }
}

pub fn diff_command(ctx: &Ctx, args: &DiffArgs) -> Attempt {
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
                let from_tree = ctx.repo.revparse_single(from)?.peel_to_tree()?;
                let to_tree = ctx.repo.revparse_single(to)?.peel_to_tree()?;

                ctx.repo
                    .diff_tree_to_tree(Some(&from_tree), Some(&to_tree), diff_options)?
            }
            (DiffPoint::Current, DiffPoint::Current) => {
                return fail("Cannot diff current to current.");
            }
            (DiffPoint::Current, DiffPoint::Ref(_)) => {
                return fail("Cannot diff in this direction.")
            }
            (DiffPoint::Ref(from), DiffPoint::Current) => {
                let from_tree = ctx.repo.revparse_single(from)?.peel_to_tree()?;

                ctx.repo
                    .diff_tree_to_workdir(Some(&from_tree), diff_options)?
            }
        },
    };

    collapse_renames(&mut diff)?;

    let mut output = OutputTarget::new()?;

    diff.print(git2::DiffFormat::Patch, |_, _, line| {
        let origin = line.origin();
        let (color_code, background_code, clear_code) = match (ctx.color_enabled(), origin) {
            (true, '+') => ("\x1b[32m", "\x1b[42m", "\x1b[0m"),
            (true, '-') => ("\x1b[31m", "\x1b[41m", "\x1b[0m"),
            _ => ("", "", ""),
        };

        let char = match origin {
            '+' => "+",
            '-' => "-",
            ' ' => " ",
            _ => "",
        };

        let line = String::from_utf8_lossy(line.content());

        let visible_line = line.trim_end();
        let trailing_whitespace = &line[visible_line.len()..];
        let trailing_non_newline = trailing_whitespace.trim_end_matches('\n');

        writeln!(
            output,
            "{color_code}{char}{visible_line}{background_code}{trailing_non_newline}{clear_code}"
        )
        .unwrap();

        true
    })?;

    output.finish();

    Ok(())
}
