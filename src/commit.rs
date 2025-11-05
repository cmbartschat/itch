use anyhow::bail;
use gix::Commit;

use crate::{ctx::Ctx, error::Maybe};

pub fn count_commits_since(_ctx: &Ctx, older: &Commit, newer: &Commit) -> Maybe<usize> {
    let mut count: usize = 0;
    let mut current = newer.clone();
    while current.id() != older.id() {
        let next = current.parent_ids().next();
        match next {
            Some(c) => {
                count += 1;
                current = c.object()?.try_into_commit()?;
            }
            None => bail!("Unable to navigate to fork point."),
        }
    }

    Ok(count)
}
