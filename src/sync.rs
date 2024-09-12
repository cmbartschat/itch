use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum ResolutionChoice {
    Incoming,
    Base,
    Later,
    Manual(String),
}

pub type ResolutionMap = HashMap<String, ResolutionChoice>;

pub struct MergeConflict {
    pub path: String,
    pub main_content: String,
    pub branch_content: String,
    pub merge_content: String,
}

pub enum Conflict {
    MainDeletion(String),
    BranchDeletion(String),
    Merge(MergeConflict),
    OpaqueMerge(String),
}

pub enum SyncDetails {
    Complete,
    Conflicted(Vec<Conflict>),
}
