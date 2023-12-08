use std::{collections::HashMap, rc::Rc};

use git2::{Commit, DiffOptions, Error};

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
    dirty: bool,
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
            format!("<[{}]", final_message.trim())
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
    let head_name = format!("{}{}", info.head.name, if info.dirty { "*" } else { "" });
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

    let head_branch = ctx.repo.find_branch(&head_name, git2::BranchType::Local)?;

    let is_head = head_branch.is_head();

    let head_commit = head_branch.into_reference().peel_to_commit()?;
    let fork_point = ctx
        .repo
        .find_commit(ctx.repo.merge_base(base_commit.id(), head_commit.id())?)?;

    let base_past_fork = count_commits_since(ctx, &fork_point, &base_commit)?;
    let head_past_fork = count_commits_since(ctx, &fork_point, &head_commit)?;

    let mut options = DiffOptions::new();

    options.include_untracked(true).include_typechange(true);

    let old_index = fork_point.tree()?;
    let new_index = head_commit.tree()?;

    struct Status {
        status: Option<char>,
        unsaved: Option<char>,
    }

    let mut statuses: HashMap<String, Status> = HashMap::new();

    let committed_diff =
        ctx.repo
            .diff_tree_to_tree(Some(&old_index), Some(&new_index), Some(&mut options))?;

    let has_saved = committed_diff.deltas().len() > 0;

    committed_diff.deltas().for_each(|d| {
        if let Some((path, status)) = match (d.old_file().exists(), d.new_file().exists()) {
            (false, false) => None,
            (false, true) => {
                let path = d.new_file().path().unwrap().to_string_lossy().into_owned();
                Some((path, 'A'))
            }
            (true, false) => {
                let path = d.old_file().path().unwrap().to_string_lossy().into_owned();
                Some((path, 'D'))
            }
            (true, true) => {
                let path = d.new_file().path().unwrap().to_string_lossy().into_owned();
                Some((path, 'M'))
            }
        } {
            statuses.insert(
                path,
                Status {
                    status: Some(status),
                    unsaved: None,
                },
            );
        }
    });

    let mut head_dirty = false;

    if is_head {
        let unsaved_diff = ctx
            .repo
            .diff_tree_to_workdir(Some(&new_index), Some(&mut options))?;

        head_dirty = unsaved_diff.deltas().len() > 0;

        unsaved_diff.deltas().for_each(|d| {
            if let Some((path, status)) = match (d.old_file().exists(), d.new_file().exists()) {
                (false, false) => None,
                (false, true) => {
                    let path = d.new_file().path().unwrap().to_string_lossy().into_owned();
                    Some((path, 'A'))
                }
                (true, false) => {
                    let path = d.old_file().path().unwrap().to_string_lossy().into_owned();
                    Some((path, 'D'))
                }
                (true, true) => {
                    let path = d.new_file().path().unwrap().to_string_lossy().into_owned();
                    Some((path, 'M'))
                }
            } {
                let existing = statuses.get(&path);
                statuses.insert(
                    path,
                    Status {
                        status: if let Some(e) = existing {
                            e.status
                        } else {
                            None
                        },
                        unsaved: Some(status),
                    },
                );
            }
        });
    }

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
        dirty: head_dirty,
    });

    if statuses.len() > 0 {
        println!("");

        let mut changes = statuses.into_iter().collect::<Vec<(String, Status)>>();
        changes.sort_by_key(|f| f.0.clone());
        changes.into_iter().for_each(|e| {
            let saved = e.1.status.unwrap_or(' ');
            let unsaved = e.1.unsaved.unwrap_or(' ');
            match (has_saved, head_dirty) {
                (true, true) => {
                    let divider = if saved == ' ' || unsaved == ' ' {
                        ' '
                    } else {
                        '.'
                    };
                    print!("{}{}{}", saved, divider, unsaved)
                }
                (true, false) => print!("{}", saved),
                (false, true) => print!("{}", unsaved),
                (false, false) => {}
            }

            println!(" {}", e.0);
        });
    }

    Ok(())
}
