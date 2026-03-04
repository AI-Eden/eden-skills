pub(crate) mod common;
mod config_ops;
mod diagnose;
mod install;
mod plan_cmd;
mod reconcile;
mod remove;
mod update;

pub use config_ops::*;
pub use diagnose::*;
pub use install::*;
pub use plan_cmd::*;
pub use reconcile::*;
pub use remove::*;
pub use update::*;

use eden_skills_core::config::InstallMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommandOptions {
    pub strict: bool,
    pub json: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateRequest {
    pub config_path: String,
    pub concurrency: Option<usize>,
    pub options: CommandOptions,
}

#[derive(Debug, Clone)]
pub struct InstallRequest {
    pub config_path: String,
    pub source: String,
    pub id: Option<String>,
    pub r#ref: Option<String>,
    pub skill: Vec<String>,
    pub all: bool,
    pub yes: bool,
    pub list: bool,
    pub version: Option<String>,
    pub registry: Option<String>,
    pub target: Vec<String>,
    pub dry_run: bool,
    pub copy: bool,
    pub options: CommandOptions,
}

#[derive(Debug, Clone)]
pub struct AddRequest {
    pub config_path: String,
    pub id: String,
    pub repo: String,
    pub r#ref: String,
    pub subpath: String,
    pub mode: InstallMode,
    pub target_specs: Vec<String>,
    pub verify_enabled: Option<bool>,
    pub verify_checks: Option<Vec<String>>,
    pub no_exec_metadata_only: Option<bool>,
    pub options: CommandOptions,
}

#[derive(Debug, Clone)]
pub struct SetRequest {
    pub config_path: String,
    pub skill_id: String,
    pub repo: Option<String>,
    pub r#ref: Option<String>,
    pub subpath: Option<String>,
    pub mode: Option<InstallMode>,
    pub verify_enabled: Option<bool>,
    pub verify_checks: Option<Vec<String>>,
    pub target_specs: Option<Vec<String>>,
    pub no_exec_metadata_only: Option<bool>,
    pub options: CommandOptions,
}
