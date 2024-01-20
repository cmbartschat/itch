use std::collections::HashMap;

#[derive(Clone)]
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
