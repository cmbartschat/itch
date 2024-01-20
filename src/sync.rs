use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum ResolutionChoice {
    Theirs,
    Yours,
    Manual(String),
}

pub type ResolutionMap = HashMap<String, ResolutionChoice>;

pub struct FullSyncArgs {
    pub names: Vec<String>,
    pub resolutions: Vec<ResolutionMap>,
}

pub struct MergeConflict {
    pub main_path: String,
    pub branch_path: String,
    pub main_content: String,
    pub branch_content: String,
    pub merge_content: String,
}

pub enum Conflict {
    MainDeletion(String),
    BranchDeletion(String),
    Merge(MergeConflict),
    OpaqueMerge(String, String),
}

pub enum SyncDetails {
    Complete,
    Conflicted(Vec<Conflict>),
}
pub type SyncResult = Vec<SyncDetails>;
