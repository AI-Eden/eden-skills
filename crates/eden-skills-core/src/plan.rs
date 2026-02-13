#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Create,
    Update,
    Noop,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanItem {
    pub skill_id: String,
    pub source_path: String,
    pub target_path: String,
    pub install_mode: String,
    pub action: Action,
    pub reasons: Vec<String>,
}
