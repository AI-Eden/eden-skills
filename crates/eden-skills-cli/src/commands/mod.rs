//! Command dispatch and shared request types for the eden-skills CLI.
//!
//! Each sub-module implements one logical command group. This module
//! re-exports all public items so callers use `commands::install_async`,
//! `commands::CommandOptions`, etc. without knowing the internal layout.

mod clean;
pub(crate) mod common;
mod config_ops;
mod diagnose;
mod docker_cmd;
mod install;
mod plan_cmd;
mod reconcile;
mod remove;
mod update;

pub use clean::*;
pub use config_ops::*;
pub use diagnose::*;
pub use docker_cmd::*;
pub use install::*;
pub use plan_cmd::*;
pub use reconcile::*;
pub use remove::*;
pub use update::*;

use eden_skills_core::config::InstallMode;

/// Flags shared by every CLI command: strict mode and JSON output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommandOptions {
    pub strict: bool,
    pub json: bool,
}

/// Parameters for the `update` command.
#[derive(Debug, Clone)]
pub struct UpdateRequest {
    pub config_path: String,
    pub concurrency: Option<usize>,
    pub apply: bool,
    pub options: CommandOptions,
}

/// Parameters for the `install` command covering URL, registry, and local modes.
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

/// Parameters for the `add` command that inserts a skill entry into config.
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

/// Parameters for the `set` command that modifies an existing skill entry.
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
