//! Domain logic layer for eden-skills.
//!
//! This crate contains configuration parsing, plan computation, source
//! sync, verification, safety analysis, lock file management, adapter
//! abstraction (local/Docker), reactor-based concurrency, registry
//! resolution, and agent discovery. It has no dependency on CLI output
//! formatting — all presentation is handled by the CLI crate.

pub mod adapter;
pub mod agents;
pub mod config;
pub mod discovery;
pub mod error;
pub mod lock;
pub mod paths;
pub mod plan;
pub mod reactor;
pub mod registry;
pub mod safety;
pub mod source;
pub mod source_format;
pub mod state;
pub mod verify;
