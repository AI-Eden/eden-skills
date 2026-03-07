//! Cache cleanup via the `clean` command and shared orphan detection helpers.
//!
//! Removes orphaned repo-cache directories under `storage/.repos` and stale
//! temporary discovery checkouts under the system temp directory.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use eden_skills_core::config::{config_dir_from_path, Config};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::resolve_path_string;
use eden_skills_core::source::repo_cache_key;

use super::common::{load_config_with_context, print_warning, remove_path, resolve_config_path};
use super::CommandOptions;
use crate::ui::{StatusSymbol, UiContext};

pub(crate) const DISCOVERY_TEMP_DIR_PREFIX: &str = "eden-skills-discovery-";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct CleanReport {
    pub(crate) dry_run: bool,
    pub(crate) removed_cache_entries: Vec<String>,
    pub(crate) removed_discovery_dirs: Vec<String>,
    pub(crate) freed_bytes: u64,
}

impl CleanReport {
    pub(crate) fn is_empty(&self) -> bool {
        self.removed_cache_entries.is_empty() && self.removed_discovery_dirs.is_empty()
    }

    pub(crate) fn nested_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "dry_run": self.dry_run,
            "removed_cache_entries": self.removed_cache_entries,
            "removed_discovery_dirs": self.removed_discovery_dirs,
            "freed_bytes": self.freed_bytes,
        })
    }

    fn command_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "action": "clean",
            "dry_run": self.dry_run,
            "removed_cache_entries": self.removed_cache_entries,
            "removed_discovery_dirs": self.removed_discovery_dirs,
            "freed_bytes": self.freed_bytes,
        })
    }
}

/// Remove orphaned repo cache entries and stale discovery directories.
///
/// # Errors
///
/// Returns [`EdenError`] when config loading, scanning, deletion, or JSON
/// serialization fails.
pub fn clean(config_path: &str, dry_run: bool, options: CommandOptions) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, options.strict)?;
    let ui = UiContext::from_env(options.json);
    for warning in loaded.warnings {
        print_warning(&ui, &warning);
    }

    let config_dir = config_dir_from_path(config_path);
    let report = clean_with_loaded_config(&loaded.config, &config_dir, dry_run)?;
    if options.json {
        print_clean_json(&report)?;
    } else {
        print_clean_summary(&ui, &report);
    }
    Ok(())
}

pub(crate) fn clean_with_loaded_config(
    config: &Config,
    config_dir: &Path,
    dry_run: bool,
) -> Result<CleanReport, EdenError> {
    let storage_root = resolve_path_string(&config.storage_root, config_dir)?;
    let orphan_cache_entries = collect_orphan_repo_cache_entries(config, &storage_root)?;
    let stale_discovery_dirs = collect_stale_discovery_dirs()?;

    let mut report = CleanReport {
        dry_run,
        ..CleanReport::default()
    };

    for path in orphan_cache_entries {
        report.freed_bytes += path_size_bytes(&path)?;
        if !dry_run {
            remove_clean_path(&path)?;
        }
        report
            .removed_cache_entries
            .push(path.display().to_string());
    }

    for path in stale_discovery_dirs {
        report.freed_bytes += path_size_bytes(&path)?;
        if !dry_run {
            remove_clean_path(&path)?;
        }
        report
            .removed_discovery_dirs
            .push(path.display().to_string());
    }

    Ok(report)
}

pub(crate) fn collect_orphan_repo_cache_entries(
    config: &Config,
    storage_root: &Path,
) -> Result<Vec<PathBuf>, EdenError> {
    let referenced = referenced_repo_cache_keys(config);
    let repo_cache_root = storage_root.join(".repos");
    let mut orphans = Vec::new();
    match fs::read_dir(&repo_cache_root) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                let file_name = entry.file_name();
                let key = file_name.to_string_lossy();
                if referenced.contains(key.as_ref()) {
                    continue;
                }
                orphans.push(path);
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(EdenError::Io(err)),
    }
    orphans.sort();
    Ok(orphans)
}

pub(crate) fn orphan_cache_target_path(path: &Path) -> String {
    let file_name = path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().to_string(),
    );
    format!(".repos/{file_name}")
}

pub(crate) fn print_clean_summary(ui: &UiContext, report: &CleanReport) {
    if report.is_empty() {
        let summary = if report.dry_run {
            "dry run complete - nothing to clean"
        } else {
            "nothing to clean"
        };
        println!(
            "{}  {} {summary}",
            ui.action_prefix("Clean"),
            ui.status_symbol(StatusSymbol::Success)
        );
        return;
    }

    if report.dry_run {
        print_clean_dry_run_summary(ui, report);
    } else {
        print_clean_apply_summary(ui, report);
    }
}

fn print_clean_apply_summary(ui: &UiContext, report: &CleanReport) {
    let mut first_line = true;
    if !report.removed_cache_entries.is_empty() {
        print_clean_count_line(
            ui,
            &mut first_line,
            report.removed_cache_entries.len(),
            "orphaned cache entry removed",
            "orphaned cache entries removed",
        );
    }
    if !report.removed_discovery_dirs.is_empty() {
        print_clean_count_line(
            ui,
            &mut first_line,
            report.removed_discovery_dirs.len(),
            "stale discovery directory removed",
            "stale discovery directories removed",
        );
    }
    println!();
    println!(
        "  {} Freed {}",
        ui.status_symbol(StatusSymbol::Success),
        format_bytes(report.freed_bytes)
    );
}

fn print_clean_dry_run_summary(ui: &UiContext, report: &CleanReport) {
    let mut first_line = true;
    if !report.removed_cache_entries.is_empty() {
        print_clean_dry_run_heading(
            ui,
            &mut first_line,
            report.removed_cache_entries.len(),
            "orphaned cache entry",
            "orphaned cache entries",
        );
        for path in &report.removed_cache_entries {
            println!("           {}", ui.styled_path(path));
        }
    }
    if !report.removed_discovery_dirs.is_empty() {
        if !first_line {
            println!();
        }
        print_clean_dry_run_heading(
            ui,
            &mut first_line,
            report.removed_discovery_dirs.len(),
            "stale discovery directory",
            "stale discovery directories",
        );
        for path in &report.removed_discovery_dirs {
            println!("           {}", ui.styled_path(path));
        }
    }
    println!();
    println!(
        "  {} Dry run complete - no files deleted",
        ui.status_symbol(StatusSymbol::Success)
    );
}

fn print_clean_count_line(
    ui: &UiContext,
    first_line: &mut bool,
    count: usize,
    singular: &str,
    plural: &str,
) {
    let prefix = if *first_line {
        *first_line = false;
        format!("{}  ", ui.action_prefix("Clean"))
    } else {
        "          ".to_string()
    };
    let label = if count == 1 { singular } else { plural };
    println!("{prefix}{count} {label}");
}

fn print_clean_dry_run_heading(
    ui: &UiContext,
    first_line: &mut bool,
    count: usize,
    singular: &str,
    plural: &str,
) {
    let prefix = if *first_line {
        *first_line = false;
        format!("{}  ", ui.action_prefix("Clean"))
    } else {
        "          ".to_string()
    };
    let label = if count == 1 { singular } else { plural };
    println!("{prefix}would remove {count} {label}:");
}

fn print_clean_json(report: &CleanReport) -> Result<(), EdenError> {
    let encoded = serde_json::to_string_pretty(&report.command_json_value())
        .map_err(|err| EdenError::Runtime(format!("failed to serialize clean json: {err}")))?;
    println!("{encoded}");
    Ok(())
}

fn referenced_repo_cache_keys(config: &Config) -> HashSet<String> {
    config
        .skills
        .iter()
        .filter(|skill| !Path::new(&skill.source.repo).is_absolute())
        .map(|skill| repo_cache_key(&skill.source.repo, &skill.source.r#ref))
        .collect()
}

fn collect_stale_discovery_dirs() -> Result<Vec<PathBuf>, EdenError> {
    let temp_root = std::env::temp_dir();
    let mut stale_dirs = Vec::new();
    match fs::read_dir(&temp_root) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy();
                if !name.starts_with(DISCOVERY_TEMP_DIR_PREFIX) {
                    continue;
                }
                if !path.is_dir() {
                    continue;
                }
                stale_dirs.push(path);
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(EdenError::Io(err)),
    }
    stale_dirs.sort();
    Ok(stale_dirs)
}

fn path_size_bytes(path: &Path) -> Result<u64, EdenError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(EdenError::Io(err)),
    };
    if metadata.file_type().is_symlink() || metadata.is_file() {
        return Ok(metadata.len());
    }
    if metadata.is_dir() {
        let mut total = 0;
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry?;
                    total += path_size_bytes(&entry.path())?;
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(err) => return Err(EdenError::Io(err)),
        }
        return Ok(total);
    }
    Ok(metadata.len())
}

fn remove_clean_path(path: &Path) -> Result<(), EdenError> {
    match remove_path(path) {
        Ok(()) => Ok(()),
        Err(EdenError::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut value = bytes as f64;
    let mut unit_index = 0usize;
    while value >= 1024.0 && unit_index + 1 < UNITS.len() {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 || value >= 10.0 {
        format!("{value:.0} {}", UNITS[unit_index])
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}
