use std::{rc::Rc, vec};

use git2::{Commit, Delta, DiffDelta, DiffFile, StatusOptions};

use crate::{
    cli::StatusArgs,
    ctx::Ctx,
    diff::{collapse_renames, good_diff_options},
    error::{fail, Attempt, Maybe},
    reset::reset_repo,
};

#[derive(Debug)]
pub struct BranchSummary {
    pub name: String,
    pub latest_message: Option<String>,
    pub commit_count: usize,
}

#[derive(Debug)]
pub struct ForkInfo {
    pub base: BranchSummary,
    pub head: BranchSummary,
    pub dirty: bool,
    pub file_statuses: Vec<SegmentedStatus>,
}

struct Styles {
    pub highlight: &'static str,
    pub muted: &'static str,
    pub end: &'static str,
}

fn get_styles(ctx: &Ctx) -> Styles {
    if ctx.color_enabled() {
        Styles {
            highlight: "\x1b[1;37m",
            muted: "\x1b[1;94m",
            end: "\x1b[0m",
        }
    } else {
        Styles {
            highlight: "",
            muted: "",
            end: "",
        }
    }
}

static DOTS_PER_BRAILLE: usize = 6;

fn create_many_dots(count: usize) -> String {
    if count > DOTS_PER_BRAILLE * 4 {
        return format!("⠿{}⠿", count);
    }

    let full_count = count / DOTS_PER_BRAILLE;
    let partial_count = count % DOTS_PER_BRAILLE;

    format!(
        "{}{}",
        "⠿".repeat(full_count),
        match partial_count {
            0 => "",
            1 => "⠁",
            2 => "⠃",
            3 => "⠇",
            4 => "⠏",
            5 => "⠟",
            _ => "x",
        }
    )
}

#[derive(Debug)]
pub struct FileStatus {
    pub from: Option<String>,
    pub to: Option<String>,
    pub status: Delta,
}

impl FileStatus {
    fn from_delta(delta: &DiffDelta) -> Self {
        Self {
            from: extract_optional_path(&delta.old_file()),
            to: extract_optional_path(&delta.new_file()),
            status: delta.status(),
        }
    }

    fn char(&self) -> char {
        match self.status {
            Delta::Unmodified => ' ',
            Delta::Added | Delta::Untracked => 'A',
            Delta::Deleted => 'D',
            Delta::Modified => 'M',
            Delta::Renamed => 'R',
            Delta::Copied => 'C',
            Delta::Typechange => 'T',
            _ => '?',
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
        Some(extract_path(d))
    } else {
        None
    }
}

#[derive(Debug)]
pub struct SegmentedStatus {
    pub committed: Option<FileStatus>,
    pub work: Option<FileStatus>,
}

impl SegmentedStatus {
    fn from_committed_delta(delta: &DiffDelta) -> Self {
        Self {
            committed: Some(FileStatus::from_delta(delta)),
            work: None,
        }
    }
    fn from_work_delta(delta: &DiffDelta) -> Self {
        Self {
            committed: None,
            work: Some(FileStatus::from_delta(delta)),
        }
    }
    fn maybe_add_work(&mut self, delta: &DiffDelta) -> bool {
        if let Some(committed) = &self.committed {
            if let Some(committed_path) = &committed.to {
                if let Some(new_base_path) = extract_optional_path(&delta.old_file()) {
                    if &new_base_path == committed_path {
                        self.work = Some(FileStatus::from_delta(delta));
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn get_work_rename_chain(&self) -> Vec<String> {
        if let Some(work) = &self.work {
            match (&work.from, &work.to) {
                (Some(from), Some(to)) => {
                    if from != to {
                        vec![from.to_string(), to.to_string()]
                    } else {
                        vec![from.to_string()]
                    }
                }
                (None, Some(to)) => vec![to.to_string()],
                (Some(from), None) => vec![from.to_string()],
                (None, None) => vec![],
            }
        } else {
            vec![]
        }
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
            if potential_rename_chain.is_empty() {
                potential_rename_chain.push(work.from);
            }
            potential_rename_chain.push(work.to);
        }

        potential_rename_chain.iter().for_each(|p| {
            if let Some(v) = p {
                rename_chain.push(v.clone());
            }
        });

        let mut rename_chain = potential_rename_chain
            .into_iter()
            .flatten()
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

fn get_post_fork_commits(info: &BranchSummary, styles: &Styles) -> String {
    let truncated_message = match &info.latest_message {
        Some(s) => {
            let mut final_message = String::from(s);
            if final_message.len() < 40 {
                final_message
            } else {
                final_message.truncate(37);
                format!("{}...", final_message.trim())
            }
        }
        _ => String::from(""),
    };

    let wrapped_message = format!(
        "{}({}{}{}){}",
        styles.highlight, styles.end, truncated_message, styles.highlight, styles.end
    );

    match info.commit_count {
        0 => "".to_string(),
        1 => wrapped_message,
        2 => format!("{}o{} ─ {wrapped_message}", styles.highlight, styles.muted,),
        3 => format!(
            "{}o{} ─ {}o{} ─ {wrapped_message}",
            styles.highlight, styles.muted, styles.highlight, styles.muted,
        ),
        _ => format!(
            "{}o{} ─ {}{}{} ─ {wrapped_message}",
            styles.highlight,
            styles.muted,
            styles.highlight,
            create_many_dots(info.commit_count - 2),
            styles.muted,
        ),
    }
}

fn draw_fork_diagram(info: &ForkInfo, styles: &Styles) {
    let base_name = &info.base.name;
    let head_name = &info.head.name;
    let head_display = get_post_fork_commits(&info.head, styles);
    let base_display = get_post_fork_commits(&info.base, styles);

    let mut main_dirty_indicator = "";

    if base_name != head_name {
        let dirty_indicator = if info.dirty { "*" } else { "" };

        if info.head.commit_count == 0 {
            println!(
                "      {}┌─{} {head_name}{dirty_indicator}",
                styles.muted, styles.end
            );
            println!("      {}↓{}", styles.muted, styles.end);
        } else {
            println!(
                "          {head_display} {}←{} {head_name}{dirty_indicator}",
                styles.muted, styles.end
            );
            println!("        {}/{}", styles.muted, styles.end);
        }
    } else if info.dirty {
        main_dirty_indicator = "*"
    }

    println!(
        "{}─ {}o{} ─ {}{base_display} {}←{} {base_name}{main_dirty_indicator}",
        styles.muted, styles.highlight, styles.muted, styles.end, styles.muted, styles.end
    )
}

fn count_commits_since(_ctx: &Ctx, older: &Commit, newer: &Commit) -> Maybe<usize> {
    let mut count: usize = 0;
    let mut current = Rc::from(newer.clone());
    while current.id() != older.id() {
        let next = current.parents().next();
        match next {
            Some(c) => {
                count += 1;
                current = Rc::from(c);
            }
            None => return fail("Unable to navigate to fork point."),
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
pub fn resolve_fork_info(ctx: &Ctx, branch_name: Option<&str>) -> Maybe<ForkInfo> {
    let repo_head = ctx.repo.head()?;
    let head_name: &str = match &branch_name {
        Some(name) => name,
        None => repo_head.shorthand().unwrap(),
    };

    let base = "main";

    let base_commit = ctx
        .repo
        .find_branch(base, git2::BranchType::Local)?
        .into_reference()
        .peel_to_commit()?;

    let head_branch = ctx.repo.find_branch(head_name, git2::BranchType::Local)?;

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
        reset_repo(ctx)?;

        let unsaved_statuses = ctx.repo.statuses(Some(
            StatusOptions::new()
                .show(git2::StatusShow::Workdir)
                .include_untracked(true)
                .recurse_untracked_dirs(true)
                .renames_index_to_workdir(true),
        ))?;

        unsaved_statuses.into_iter().for_each(|unsaved_status| {
            if let Some(d) = unsaved_status.index_to_workdir() {
                if d.status() == Delta::Ignored {
                    return;
                }
                head_dirty = true;
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
            }
        });
    }

    return Ok(ForkInfo {
        base: BranchSummary {
            name: base.to_string(),
            latest_message: base_commit.summary().map(|e| e.to_string()),
            commit_count: base_past_fork + 1,
        },
        head: BranchSummary {
            name: head_name.to_string(),
            latest_message: head_commit.summary().map(|e| e.to_string()),
            commit_count: head_past_fork,
        },
        dirty: head_dirty,
        file_statuses: statuses,
    });
}

pub fn status_command(ctx: &Ctx, args: &StatusArgs) -> Attempt {
    let info = resolve_fork_info(ctx, args.name.as_deref())?;

    let styles = get_styles(ctx);

    draw_fork_diagram(&info, &styles);

    if !info.file_statuses.is_empty() {
        println!();

        for status in info.file_statuses.into_iter() {
            status.print();
        }
    }

    Ok(())
}
