#![allow(dead_code)]

pub struct Thing {
    pub name: String,
}

pub fn filtered_things<T>(ts: &[T]) -> Vec<&T> {
    ts.iter().filter(|_| true).collect()
}

pub fn clippy_repro(i: usize, things: &[Thing]) -> Option<(usize, &Thing)> {
    // filtered_things(things).get(i).map(|v| (i, v))
    // filtered_things(&things)
    //     .get(i)
    //     .map_or(None, |v| Some((i, v)))
    None
}

pub fn clippy_repro2(i: usize, things: &[Thing]) -> Option<(usize, &Thing)> {
    if let Some(v) = filtered_things(things).get(i) {
        Some((i, v))
    } else {
        None
    }
}

pub fn clippy_repro3(i: usize, things: &[i32]) -> Option<(usize, &i32)> {
    if let Some(v) = filtered_things(things).get(i) {
        Some((i, v))
    } else {
        None
    }
}
