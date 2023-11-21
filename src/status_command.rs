use std::rc::Rc;

use git2::{Commit, Error};
use log::debug;

use crate::{base::resolve_base, cli::StatusArgs, ctx::Ctx};

#[derive(Debug)]
struct BranchSummary<'a> {
    name: &'a str,
    latest_message: Option<&'a str>,
    commit_count: usize,
}

#[derive(Debug)]
struct ForkInfo<'a> {
    base: BranchSummary<'a>,
    head: BranchSummary<'a>,
}

/*
         o -- <4> - o<[save] <- example*
       /
  ... o - o - <17> - o<[break something] <- main
*/

/*
      ┌─ o ─ <4> - o<[save] <- example*
      ↓
  ... o ←─ main
*/

/*
        ┌─  example
        ↓
  ─ o ─ o ← main
*/

fn get_post_fork_commits(info: &BranchSummary) -> String {
    match info.commit_count {
        0 => "".to_string(),
        1 => "- o".to_string(),
        2 => "- o - o".to_string(),
        3 => "- o - o - o".to_string(),
        _ => format!("- o - <{}> - o", info.commit_count - 2),
    }
}

fn draw_fork_diagram(_info: &ForkInfo) {
    debug!("{:?}", _info);
    println!(
        "         ┌─ {} {}",
        get_post_fork_commits(&_info.head),
        _info.head.name
    );
    println!("         ↓");
    println!(
        "   ─ o ─ o{} ← {}",
        get_post_fork_commits(&_info.base),
        _info.base.name
    );
}

fn count_commits_since(_ctx: &Ctx, older: &Commit, newer: &Commit) -> Result<usize, Error> {
    debug!(
        "Counting commits between {:?} and {:?}",
        older.id(),
        newer.id()
    );
    let mut count: usize = 0;
    let mut current = Rc::from(newer.clone());
    while current.id() != older.id() {
        let next = current.parents().next();
        match next {
            Some(c) => {
                count += 1;
                current = Rc::from(c);
            }
            None => return Err(Error::from_str("Unable to navigate to fork point.")),
        }
    }

    Ok(count)
}

fn _status_command(ctx: &Ctx, args: &StatusArgs, base: &str) -> Result<(), Error> {
    let repo_head = ctx.repo.head()?;
    let head_name: &str = match &args.target {
        Some(name) => name.as_str(),
        None => repo_head.shorthand().unwrap(),
    };

    if head_name == (String::from("refs/heads/") + &base) {
        println!("Status on base: {head_name} {base}");
    } else {
        println!("Status between {head_name} and {base}");
    }

    let base_commit = ctx
        .repo
        .find_branch(&base, git2::BranchType::Local)?
        .into_reference()
        .peel_to_commit()?;

    let head_commit = ctx
        .repo
        .find_branch(&head_name, git2::BranchType::Local)?
        .into_reference()
        .peel_to_commit()?;
    let fork_point = ctx
        .repo
        .find_commit(ctx.repo.merge_base(base_commit.id(), head_commit.id())?)?;

    let base_past_fork = count_commits_since(ctx, &fork_point, &base_commit)?;
    let head_past_fork = count_commits_since(ctx, &fork_point, &head_commit)?;

    draw_fork_diagram(&ForkInfo {
        base: BranchSummary {
            name: base,
            latest_message: base_commit.summary(),
            commit_count: base_past_fork,
        },
        head: BranchSummary {
            name: head_name,
            latest_message: head_commit.summary(),
            commit_count: head_past_fork,
        },
    });

    Ok(())
}

pub fn status_command(ctx: &Ctx, args: &StatusArgs) -> Result<(), ()> {
    let base = resolve_base(&None)?;

    // If separate branch:

    /*

    On branch: example
         o -- <4> - o<[save] <- example*
       /
    -o - o - <17> - o<[break something] <- main

    Changes:

    file1.txt

    + a
    - b
     */

    return _status_command(&ctx, &args, &base).map_err(|e| {
        debug!("{}", e);
        ()
    });
}
