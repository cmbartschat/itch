pub struct Thing {
    pub name: String,
}

pub fn filtered_things(ts: &[Thing]) -> Vec<&Thing> {
    ts.iter().filter(|t| !t.name.is_empty()).collect()
}

pub fn clippy_repro(i: usize, things: &[Thing]) -> Option<(usize, &Thing)> {
    if let Some(v) = filtered_things(things).get(i) {
        Some((i, v))
    } else {
        None
    }
}
