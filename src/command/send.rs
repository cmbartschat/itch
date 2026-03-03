use std::collections::HashSet;

use clap::builder::OsStr;
use git2::{Commit, ErrorCode, Oid};

use crate::{
    cli::SaveArgs,
    ctx::Ctx,
    diff::good_diff_options,
    editor::edit_temp_text,
    error::{Attempt, Maybe},
    fail,
    reset::reset_repo,
    save::{include_footer, resolve_commit_message, save_temp},
};

fn make_send_tag_name(base_name: &str) -> String {
    format!("{base_name}-send")
}

fn create_stacked_send_commit(
    ctx: &Ctx,
    previous: Option<&Commit>,
    message: &str,
    files_to_include: Vec<String>,
) -> Maybe<Oid> {
    let head_commit = ctx.repo.head()?.peel_to_commit()?;

    let fork_commit = ctx.repo.find_commit(
        ctx.repo.merge_base(
            head_commit.id(),
            ctx.repo
                .find_branch("main", git2::BranchType::Local)?
                .into_reference()
                .peel_to_commit()?
                .id(),
        )?,
    )?;

    let fork_tree = fork_commit.tree()?;

    let mut new_tree = ctx.repo.treebuilder(Some(&fork_tree))?;

    for file in files_to_include {
        if let Some(e) = fork_tree.get_name(file.as_str()) {
            new_tree.insert(file, e.id(), e.filemode())?;
        } else {
            eprintln!("Removing {file}");
            new_tree.remove(file)?;
        }
    }

    let signature = ctx.repo.signature()?;

    let tree = ctx.repo.find_tree(new_tree.write()?)?;

    let mut parents = vec![&fork_commit];

    if let Some(prev) = previous {
        parents.push(prev);
    }

    let oid = ctx
        .repo
        .commit(None, &signature, &signature, message, &tree, &parents)?;

    Ok(oid)
}

pub fn list_changed_files(ctx: &Ctx, head_commit: &Commit) -> Maybe<Vec<String>> {
    let fork_point = ctx.repo.find_commit(
        ctx.repo.merge_base(
            head_commit.id(),
            ctx.repo
                .find_branch("main", git2::BranchType::Local)?
                .into_reference()
                .peel_to_commit()?
                .id(),
        )?,
    )?;

    let diff = ctx.repo.diff_tree_to_tree(
        Some(&fork_point.tree()?),
        Some(&head_commit.tree()?),
        Some(&mut good_diff_options()),
    )?;

    let mut files = Vec::with_capacity(diff.deltas().len());

    for f in diff.deltas() {
        files.push(
            match (f.old_file().path(), f.new_file().path()) {
                (Some(old), Some(new)) => {
                    assert_eq!(old, new);
                    old
                }
                (Some(old), None) => old,
                (None, Some(new)) => new,
                (None, None) => return fail!("Diff is missing filename"),
            }
            .to_string_lossy()
            .to_string(),
        );
    }

    Ok(files)
}

pub fn send_command(ctx: &Ctx, args: &SaveArgs) -> Attempt {
    save_temp(ctx, "Save before sending".into())?;
    let head = ctx.repo.head()?;

    let Some(base_name) = head.shorthand() else {
        return fail!("Unable to resolve current branch name");
    };

    let tag_name = make_send_tag_name(base_name);

    let previous_push_commit = match ctx.repo.find_reference(&format!("refs/tags/{tag_name}")) {
        Ok(e) => Some(e.peel_to_commit()?),
        Err(e) if e.code() == ErrorCode::NotFound => None,
        Err(e) => return Err(e.into()),
    };

    let files_to_include = if let Some(e) = &previous_push_commit {
        list_changed_files(ctx, e)?
    } else {
        let all_files = list_changed_files(ctx, &head.peel_to_commit()?)?;
        let mut content = String::from("Delete paths you do not wish to send:\n");
        for file in &all_files {
            content.push('\n');
            content.push_str(file);
        }
        let extension = OsStr::from("txt");
        let updated_text = edit_temp_text(content.as_str(), Some(&extension))?;
        let updated = updated_text.lines().collect::<HashSet<_>>();
        all_files
            .into_iter()
            .filter(|f| updated.contains(f.as_str()))
            .collect()
    };

    let message = match (
        resolve_commit_message(&args.message),
        previous_push_commit.as_ref().and_then(|e| e.message()),
    ) {
        (Some(message), _) => include_footer(ctx, &message)?,
        (None, Some(e)) => e.to_string(),
        (None, None) => include_footer(ctx, "Send")?,
    };

    let new_commit = create_stacked_send_commit(
        ctx,
        previous_push_commit.as_ref(),
        &message,
        files_to_include,
    )?;

    ctx.repo
        .tag_lightweight(&tag_name, &ctx.repo.find_object(new_commit, None)?, true)?;

    reset_repo(ctx)?;
    Ok(())
}

pub fn resend_command(_ctx: &Ctx, _args: &SaveArgs) -> Attempt {
    todo!()
}

pub fn unsend_command(_ctx: &Ctx) -> Attempt {
    todo!()
}
