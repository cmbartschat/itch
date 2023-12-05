use std::rc::Rc;

use git2::{Commit, Error};

use crate::{cli::StatusArgs, ctx::Ctx};

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
==========================================

        o ─ <4> ─ o<[save] ←─ example*
       /
─ o ─ o ←─ main

==========================================

         ┌ o ─ <4> ─ o<[save] ←─ example*
         │
─ o ─ o ─┴──── main

==========================================

      ┌─  example
      │
─ o ─ o ←─ main

==========================================

      ┌─ example
      │
─ o ─ o<[message1] ←─ main

==========================================

─ o ─ o<[message1] ←─ main

==========================================

      ┌─ o<[message2] (example)
      │
─ o ─ o<[message1] (main)

==========================================

                         ┌─ example
                         ↓
─ o ─ o<[message1] ─ o ─ o<[message2]
      ↑
      └─ main

==========================================

      o ─ o<[message2] ←─ example
      │
─ o ─ o<[message1] ←─ main

==========================================

      o<[message2] ←─ example
      │
─ o ─ o<[message1] ←─ main

==========================================

      ┌─ example
      ↓
─ o ─ o<[message1] ←─ main

==========================================
*/

fn get_post_fork_commits(info: &BranchSummary) -> String {
    let message_part = match info.latest_message {
        Some(s) => {
            let mut final_message = String::from(s);
            final_message.truncate(25);
            format!("<[{}]", final_message)
        }
        _ => String::from(""),
    };

    match info.commit_count {
        0 => "".to_string(),
        1 => format!("o{}", message_part),
        2 => format!("o ─ o{}", message_part),
        3 => format!("o ─ o ─ o{}", message_part),
        _ => format!("o ─ <{}> ─ o{}", info.commit_count - 2, message_part),
    }
}

fn draw_fork_diagram(info: &ForkInfo) {
    let base_name = info.base.name;
    let head_name = info.head.name;
    let head_display = get_post_fork_commits(&info.head);
    let base_display = get_post_fork_commits(&info.base);

    if base_name != head_name {
        if info.head.commit_count == 0 {
            println!("      ┌─ {head_name}");
            println!("      ↓");
        } else {
            println!("          {head_display} ← {head_name}");
            println!("        /");
        }
    }

    println!("─ o ─ {base_display} ← {base_name}")
}

fn count_commits_since(_ctx: &Ctx, older: &Commit, newer: &Commit) -> Result<usize, Error> {
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
pub fn status_command(ctx: &Ctx, args: &StatusArgs) -> Result<(), Error> {
    let repo_head = ctx.repo.head()?;
    let head_name: &str = match &args.target {
        Some(name) => name.as_str(),
        None => repo_head.shorthand().unwrap(),
    };

    let base = "main";

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
            commit_count: base_past_fork + 1,
        },
        head: BranchSummary {
            name: head_name,
            latest_message: head_commit.summary(),
            commit_count: head_past_fork,
        },
    });

    Ok(())
}
