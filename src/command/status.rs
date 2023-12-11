use std::rc::Rc;

use git2::{Commit, DiffDelta, DiffFile, Error};

use crate::{
    cli::StatusArgs,
    ctx::Ctx,
    diff::{collapse_renames, good_diff_options},
};

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

#[derive(Debug)]
struct FileStatus {
    from: Option<String>,
    to: Option<String>,
    changed: bool,
}

impl FileStatus {
    fn from_delta(delta: &DiffDelta) -> Self {
        Self {
            from: extract_optional_path(&delta.old_file()),
            to: extract_optional_path(&delta.new_file()),
            changed: delta.old_file().id() != delta.new_file().id(),
        }
    }

    fn char(&self) -> char {
        match (&self.from, &self.to) {
            (None, None) => ' ',
            (None, Some(_)) => 'A',
            (Some(_), None) => 'D',
            (Some(a), Some(b)) => {
                if self.changed {
                    'M'
                } else if a != b {
                    'R'
                } else {
                    ' '
                }
            }
        }
    }
}

fn extract_path(d: &DiffFile) -> String {
    d.path()
        .expect("Expected path for DiffFile.")
        .to_string_lossy()
        .into_owned()
}

fn extract_optional_path(d: &DiffFile) -> Option<String> {
    if d.exists() {
        Some(extract_path(&d))
    } else {
        None
    }
}

#[derive(Debug)]
struct SegmentedStatus {
    committed: Option<FileStatus>,
    work: Option<FileStatus>,
}

impl SegmentedStatus {
    fn from_committed_delta(delta: &DiffDelta) -> Self {
        return Self {
            committed: Some(FileStatus::from_delta(&delta)),
            work: None,
        };
    }
    fn from_work_delta(delta: &DiffDelta) -> Self {
        return Self {
            committed: None,
            work: Some(FileStatus::from_delta(&delta)),
        };
    }
    fn maybe_add_work(&mut self, delta: &DiffDelta) -> bool {
        if let Some(committed) = &self.committed {
            if let Some(committed_path) = &committed.to {
                if let Some(new_base_path) = extract_optional_path(&delta.old_file()) {
                    if &new_base_path == committed_path {
                        self.work = Some(FileStatus::from_delta(&delta));
                        return true;
                    }
                }
            }
        }

        return false;
    }
    fn print(self) {
        let mut committed_char = ' ';
        let mut work_char = ' ';

        let mut rename_chain: Vec<String> = Vec::new();

        let mut potential_rename_chain: Vec<Option<String>> = Vec::new();

        if let Some(committed) = self.committed {
            committed_char = committed.char();
            potential_rename_chain.push(committed.from);
            potential_rename_chain.push(committed.to);
        }
        if let Some(work) = self.work {
            work_char = work.char();
            potential_rename_chain.push(work.to);
        }

        potential_rename_chain.iter().for_each(|p| match p {
            Some(v) => {
                rename_chain.push(v.clone());
            }
            _ => {}
        });

        let mut rename_chain = potential_rename_chain
            .into_iter()
            .filter_map(|f| f)
            .collect::<Vec<String>>();

        if let Some(first) = rename_chain.first() {
            if rename_chain.iter().skip(1).all(|f| f == first) {
                rename_chain.truncate(1);
            }
        }

        let combined = if committed_char == ' ' || work_char == ' ' {
            ' '
        } else {
            '.'
        };

        println!(
            "{}{}{} {}",
            committed_char,
            combined,
            work_char,
            rename_chain.join(" -> ")
        );
    }
}

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
    let head_name = info.head.name;
    let head_display = get_post_fork_commits(&info.head);
    let base_display = get_post_fork_commits(&info.base);

    let mut main_dirty_indicator = "";

    if base_name != head_name {
        let dirty_indicator = if info.dirty { "*" } else { "" };

        if info.head.commit_count == 0 {
            println!("      ┌─ {head_name}{dirty_indicator}");
            println!("      ↓");
        } else {
            println!("          {head_display} ← {head_name}{dirty_indicator}");
            println!("        /");
        }
    } else if info.dirty {
        main_dirty_indicator = "*"
    }

    println!("─ o ─ {base_display} ← {base_name}{main_dirty_indicator}")
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

    let mut options = good_diff_options();

    let old_index = fork_point.tree()?;
    let new_index = head_commit.tree()?;

    let mut statuses: Vec<SegmentedStatus> = vec![];

    let mut committed_diff =
        ctx.repo
            .diff_tree_to_tree(Some(&old_index), Some(&new_index), Some(&mut options))?;

    collapse_renames(&mut committed_diff)?;

    let _has_saved = committed_diff.deltas().len() > 0;

    committed_diff.deltas().for_each(|d| {
        statuses.push(SegmentedStatus::from_committed_delta(&d));
    });

    let mut head_dirty = false;

    if is_head {
        let mut unsaved_diff = ctx
            .repo
            .diff_tree_to_workdir(Some(&new_index), Some(&mut options))?;

        collapse_renames(&mut unsaved_diff)?;

        head_dirty = unsaved_diff.deltas().len() > 0;

        unsaved_diff.deltas().for_each(|d| {
            let mut found = false;

            for change in statuses.iter_mut() {
                if change.maybe_add_work(&d) {
                    found = true;
                    break;
                }
            }

            if !found {
                statuses.push(SegmentedStatus::from_work_delta(&d));
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

        for status in statuses.into_iter() {
            status.print();
        }
    }

    Ok(())
}
