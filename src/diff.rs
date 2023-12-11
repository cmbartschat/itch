use git2::{Diff, DiffFindOptions, DiffOptions};

pub fn collapse_renames(diff: &mut Diff) -> Result<(), git2::Error> {
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
