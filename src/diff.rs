use git2::{Blob, Diff, DiffFindOptions, DiffLine, DiffOptions, Oid, Repository};
use std::io::Read;

use crate::error::{fail, Attempt, Maybe};

pub fn split_diff_line(line: &DiffLine) -> (String, String) {
    let line = String::from_utf8_lossy(line.content());

    let visible_line = line.trim_end();
    let trailing_whitespace = &line[visible_line.len()..];
    let trailing_non_newline = trailing_whitespace.trim_end_matches('\n');
    (visible_line.to_string(), trailing_non_newline.to_string())
}

pub fn collapse_renames(diff: &mut Diff) -> Attempt {
    let mut options = DiffFindOptions::new();
    options.all(true);
    options.break_rewrites(false);
    diff.find_similar(Some(&mut options))
}

pub fn good_diff_options() -> DiffOptions {
    let mut options = DiffOptions::new();
    options.include_typechange(true);
    options.show_untracked_content(true);
    options.recurse_untracked_dirs(true);
    options
}

fn oid_to_string<'a>(repo: &'a Repository, oid: &git2::Oid) -> Maybe<(Option<Blob<'a>>, String)> {
    if oid.is_zero() {
        return Ok((None, "".to_string()));
    }
    let blob = repo.find_blob(*oid)?;
    if blob.is_binary() {
        return fail("Cannot load binary data line-by-line");
    }
    let mut original_content = String::new();
    blob.content()
        .read_to_string(&mut original_content)
        .unwrap();
    Ok((Some(blob), original_content))
}

fn get_lines(str: &str) -> Vec<&str> {
    let mut original_lines: Vec<&str> = str.split_inclusive("\n").collect();
    if original_lines.last() == Some(&"") {
        original_lines.pop();
    }
    original_lines
}

fn copy_lines(target: &mut String, source: &[&str], range: &Range) {
    for line in &source[range.0..range.1] {
        target.push_str(line);
    }
}

#[derive(Debug, Clone)]

struct Range(usize, usize);

impl Range {
    #[must_use]
    pub fn from_indices(start: u32, lines: u32) -> Self {
        if lines == 0 {
            let index = start as usize;
            return Self(index, index);
        }

        Self(start as usize - 1, start as usize + lines as usize - 1)
    }

    #[must_use]
    pub fn touches(&self, other: &Self) -> bool {
        let is_past_left = self.1 + 1 < other.0;
        let is_past_right = other.1 + 1 < self.0;

        !(is_past_left || is_past_right)
    }

    #[must_use]
    pub fn join(&self, other: &Self) -> Self {
        Self(self.0.min(other.0), self.1.max(other.1))
    }

    #[must_use]
    pub fn join_with_opt(&self, other: &Option<Self>) -> Self {
        if let Some(other) = other {
            self.join(other)
        } else {
            self.clone()
        }
    }

    pub fn join_mut(&mut self, other: &Self) {
        self.0 = self.0.min(other.0);
        self.1 = self.1.max(other.1);
    }
}

#[derive(Debug)]
struct MyHunk {
    old: Range,
    new: Range,
}

fn get_diff_hunks(
    repo: &Repository,
    old_blob: &Option<git2::Blob>,
    new_blob: &Option<git2::Blob>,
) -> Vec<MyHunk> {
    let old_as_path = None;
    let new_as_path = None;
    let mut opts = DiffOptions::new();
    opts.context_lines(0);
    opts.ignore_whitespace(true);
    let file_cb = None;
    let binary_cb = None;
    let line_cb = None;

    let mut res = vec![];

    repo.diff_blobs(
        old_blob.as_ref(),
        old_as_path,
        new_blob.as_ref(),
        new_as_path,
        Some(&mut opts),
        file_cb,
        binary_cb,
        Some(&mut |_, hunk| {
            res.push(MyHunk {
                old: Range::from_indices(hunk.old_start(), hunk.old_lines()),
                new: Range::from_indices(hunk.new_start(), hunk.new_lines()),
            });
            true
        }),
        line_cb,
    )
    .unwrap();

    res
}

pub fn get_merge_text(
    repo: &Repository,
    original_id: &Oid,
    upstream_id: &Oid,
    branch_id: &Oid,
) -> Maybe<String> {
    let (original_blob, original_string) = oid_to_string(repo, original_id)?;
    let original_lines = get_lines(&original_string);

    let (upstream_blob, upstream_string) = oid_to_string(repo, upstream_id)?;
    let upstream_lines = get_lines(&upstream_string);

    let (branch_blob, branch_string) = oid_to_string(repo, branch_id)?;
    let branch_lines = get_lines(&branch_string);

    let mut branch_hunks = get_diff_hunks(repo, &original_blob, &branch_blob)
        .into_iter()
        .peekable();
    let mut upstream_hunks = get_diff_hunks(repo, &original_blob, &upstream_blob)
        .into_iter()
        .peekable();

    let mut original_index: usize = 0;

    let mut res = String::new();

    for _ in 0..1_000 {
        let mut original_range: Range = match (upstream_hunks.peek(), branch_hunks.peek()) {
            (Some(upstream), Some(branch)) => {
                if upstream.old.0 < branch.old.0 {
                    upstream.old.clone()
                } else {
                    branch.old.clone()
                }
            }
            (Some(upstream), None) => upstream.old.clone(),
            (None, Some(branch)) => branch.old.clone(),
            (None, None) => break,
        };

        let mut upstream_range: Option<Range> = None;
        let mut branch_range: Option<Range> = None;

        let mut should_continue = true;
        while should_continue {
            should_continue = false;

            if let Some(next_upstream) = upstream_hunks.peek() {
                if next_upstream.old.touches(&original_range) {
                    should_continue = true;
                    original_range.join_mut(&next_upstream.old);
                    upstream_range = Some(next_upstream.new.join_with_opt(&upstream_range));
                    upstream_hunks.next();
                }
            }

            if let Some(next_branch) = branch_hunks.peek() {
                if next_branch.old.touches(&original_range) {
                    should_continue = true;
                    original_range.join_mut(&next_branch.old);
                    branch_range = Some(next_branch.new.join_with_opt(&branch_range));
                    branch_hunks.next();
                }
            }
        }

        for line in &original_lines[original_index..original_range.0] {
            res.push_str(line);
        }

        match (upstream_range, branch_range) {
            (Some(upstream_range), Some(branch_range)) => {
                res.push_str("<<<<<<<\n");
                copy_lines(&mut res, &upstream_lines, &upstream_range);
                res.push_str("=======\n");
                copy_lines(&mut res, &branch_lines, &branch_range);
                res.push_str(">>>>>>>\n");
            }
            (Some(upstream_range), None) => {
                copy_lines(&mut res, &upstream_lines, &upstream_range);
            }
            (None, Some(branch_range)) => {
                copy_lines(&mut res, &branch_lines, &branch_range);
            }
            (None, None) => {
                panic!("Should always have upstream or branch range.");
            }
        };

        original_index = original_range.1;
    }

    {
        for line in &original_lines[original_index..] {
            res.push_str(line);
        }
    }

    Ok(res)
}

#[cfg(test)]
mod merge_tests {
    use git2::{Oid, Repository};
    use tempfile::TempDir;

    use crate::diff::get_merge_text;

    pub fn init_repo() -> (TempDir, Repository) {
        let dir = tempfile::tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        (dir, repo)
    }

    fn merge_files(original: &str, upstream: &str, branch: &str) -> String {
        let (dir, repo) = init_repo();
        let original_id = repo.blob(original.as_bytes()).unwrap();
        let upstream_id = repo.blob(upstream.as_bytes()).unwrap();
        let branch_id = repo.blob(branch.as_bytes()).unwrap();
        let res = get_merge_text(&repo, &original_id, &upstream_id, &branch_id).unwrap();
        drop(dir);
        res
    }

    #[test]
    fn equal_files() {
        let combined = merge_files("same\n", "same\n", "same\n");
        assert_eq!(&combined, "same\n");
    }

    #[test]
    fn delete_line() {
        let combined = merge_files("same\n", "same\n", "\n");
        assert_eq!(&combined, "\n");
    }

    #[test]
    fn add_line() {
        let combined = merge_files("a\nb\nc\n", "a\nb\nc\n", "a\nb\nc\nnew\n");
        assert_eq!(&combined, "a\nb\nc\nnew\n");
    }

    #[test]
    fn add_line_upstream() {
        let combined = merge_files("same\n", "same\nupstream\n", "same\n");
        assert_eq!(&combined, "same\nupstream\n");
    }

    #[test]
    fn add_line_within() {
        let combined = merge_files(
            "\
a
b
c\n",
            "\
a
b
c\n",
            "\
a
b
branch
c\n",
        );
        assert_eq!(
            &combined,
            "\
a
b
branch
c\n"
        );
    }

    #[test]
    fn add_lines_separately() {
        let combined = merge_files(
            "\
a
b
c\n",
            "\
upstream
a
b
c\n",
            "\
a
b
branch
c\n",
        );
        assert_eq!(
            &combined,
            "\
upstream
a
b
branch
c\n"
        );
    }

    #[test]
    fn remove_line_within() {
        let combined = merge_files(
            "\
    a
    b
    c\n",
            "\
    a
    b
    c\n",
            "\
    a
    c\n",
        );
        assert_eq!(
            &combined,
            "\
    a
    c\n"
        );
    }

    #[test]
    fn multiple_overlapping() {
        let combined = merge_files(
            "\
a
b
c
d1
d2
d3
e
f
g\n",
            "\
a
b
f
g\n",
            "\
a
b
c2
d1
d2
d3
e2
f
g\n",
        );
        assert_eq!(
            &combined,
            "\
a
b
<<<<<<<
=======
c2
d1
d2
d3
e2
>>>>>>>
f
g\n"
        );
    }

    #[test]
    fn basic_ignore_whitespace() {
        let original = r"
    return 1;
";
        let upstream = r"
return 2;
";
        let branch = r"
return 1;
";
        let result = r"
return 2;
";
        let combined = merge_files(original, upstream, branch);
        assert_eq!(&combined, result);
    }

    #[test]
    fn add_conflicting_line() {
        let combined = merge_files("same\n", "same\nupstream\n", "same\nbranch\n");
        assert_eq!(
            &combined,
            "same\n<<<<<<<\nupstream\n=======\nbranch\n>>>>>>>\n"
        );
    }

    #[test]
    fn no_original() {
        let (dir, repo) = init_repo();
        let original_id = Oid::zero();
        let upstream_id = repo.blob("upstream content\n".as_bytes()).unwrap();
        let branch_id = repo.blob("branch content\n".as_bytes()).unwrap();
        let combined = get_merge_text(&repo, &original_id, &upstream_id, &branch_id).unwrap();
        assert_eq!(
            &combined,
            "<<<<<<<\nupstream content\n=======\nbranch content\n>>>>>>>\n"
        );
        drop(dir);
    }

    #[test]
    fn no_branch() {
        let (dir, repo) = init_repo();
        let original_id = repo.blob("original content\n".as_bytes()).unwrap();
        let upstream_id = repo.blob("upstream content\n".as_bytes()).unwrap();
        let branch_id = Oid::zero();
        let combined = get_merge_text(&repo, &original_id, &upstream_id, &branch_id).unwrap();
        assert_eq!(&combined, "<<<<<<<\nupstream content\n=======\n>>>>>>>\n");
        drop(dir);
    }

    #[test]
    fn no_upstream() {
        let (dir, repo) = init_repo();
        let original_id = repo.blob("original content\n".as_bytes()).unwrap();
        let upstream_id = Oid::zero();
        let branch_id = repo.blob("branch content\n".as_bytes()).unwrap();
        let combined = get_merge_text(&repo, &original_id, &upstream_id, &branch_id).unwrap();
        assert_eq!(&combined, "<<<<<<<\n=======\nbranch content\n>>>>>>>\n");
        drop(dir);
    }
}

#[cfg(test)]
mod range_tests {
    use super::Range;

    #[test]
    fn touch_venn() {
        assert!(Range(0, 60).touches(&Range(40, 100)));
    }

    #[test]
    fn touch_reverse_venn() {
        assert!(Range(40, 100).touches(&Range(0, 60)));
    }

    #[test]
    fn touch_outside() {
        assert!(Range(0, 100).touches(&Range(40, 60)));
    }

    #[test]
    fn touch_inside() {
        assert!(Range(40, 60).touches(&Range(0, 100)));
    }

    #[test]
    fn touch_direct() {
        assert!(Range(0, 10).touches(&Range(10, 100)));
    }

    #[test]
    fn touch_reverse_direct() {
        assert!(Range(10, 100).touches(&Range(0, 10)));
    }

    #[test]
    fn touch_separate() {
        assert!(!Range(0, 20).touches(&Range(25, 100)));
    }

    #[test]
    fn touch_reverse_separate() {
        assert!(!Range(25, 100).touches(&Range(0, 20)));
    }
}
