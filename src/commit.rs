use std::rc::Rc;

use git2::Commit;

use crate::{
    ctx::Ctx,
    error::{Maybe, fail},
};

pub fn count_commits_since(_ctx: &Ctx, older: &Commit, newer: &Commit) -> Maybe<usize> {
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
