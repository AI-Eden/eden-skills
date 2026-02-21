use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::EdenError;
use crate::paths::resolve_target_path;

pub const LOCK_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockFile {
    pub version: u32,
    #[serde(default)]
    pub skills: Vec<LockSkillEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockSkillEntry {
    pub id: String,
    pub source_repo: String,
    pub source_subpath: String,
    pub source_ref: String,
    #[serde(default)]
    pub resolved_commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_version: Option<String>,
    pub install_mode: String,
    pub installed_at: String,
    pub targets: Vec<LockTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockTarget {
    pub agent: String,
    pub path: String,
}

impl LockFile {
    pub fn empty() -> Self {
        Self {
            version: LOCK_VERSION,
            skills: Vec::new(),
        }
    }
}

/// Derive lock file path from config file path.
/// Replaces `.toml` extension with `.lock`; appends `.lock` otherwise.
pub fn lock_path_for_config(config_path: &Path) -> PathBuf {
    if config_path.extension().is_some_and(|ext| ext == "toml") {
        return config_path.with_extension("lock");
    }
    let mut name = config_path.as_os_str().to_owned();
    name.push(".lock");
    PathBuf::from(name)
}

/// Read and parse a lock file. Returns `Ok(None)` when the file is missing
/// (LCK-005) or corrupted/version-mismatched (LCK-006, warning emitted).
pub fn read_lock_file(lock_path: &Path) -> Result<Option<LockFile>, EdenError> {
    let content = match std::fs::read_to_string(lock_path) {
        Ok(c) => c,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(EdenError::Io(err)),
    };

    match toml::from_str::<LockFile>(&content) {
        Ok(lock) if lock.version != LOCK_VERSION => {
            eprintln!(
                "warning: skills.lock has unsupported version {}; performing full reconciliation",
                lock.version
            );
            Ok(None)
        }
        Ok(lock) => Ok(Some(lock)),
        Err(_) => {
            eprintln!("warning: skills.lock is corrupted; performing full reconciliation");
            Ok(None)
        }
    }
}

/// Serialize and write a lock file. Entries are sorted alphabetically by id;
/// targets within each entry are sorted by agent (LCK-009).
pub fn write_lock_file(lock_path: &Path, lock: &LockFile) -> Result<(), EdenError> {
    let mut sorted = lock.clone();
    sorted.skills.sort_by(|a, b| a.id.cmp(&b.id));
    for skill in &mut sorted.skills {
        skill.targets.sort_by(|a, b| a.agent.cmp(&b.agent));
    }

    let content = toml::to_string_pretty(&sorted)
        .map_err(|err| EdenError::Runtime(format!("failed to serialize lock file: {err}")))?;
    std::fs::write(lock_path, content)?;
    Ok(())
}

/// Build a lock file snapshot from the current config and resolved state.
/// `resolved_commits` maps skill id to resolved SHA-1 (empty string if unavailable).
pub fn build_lock_from_config(
    config: &Config,
    config_dir: &Path,
    resolved_commits: &std::collections::HashMap<String, String>,
) -> Result<LockFile, EdenError> {
    let now = utc_now_iso8601();
    let mut entries = Vec::with_capacity(config.skills.len());

    for skill in &config.skills {
        let mut targets = Vec::with_capacity(skill.targets.len());
        for target in &skill.targets {
            let target_root = resolve_target_path(target, config_dir)?;
            let target_path = target_root.join(&skill.id);
            targets.push(LockTarget {
                agent: target.agent.as_str().to_string(),
                path: target_path.display().to_string(),
            });
        }

        let resolved_commit = resolved_commits.get(&skill.id).cloned().unwrap_or_default();

        let resolved_version = if crate::config::is_registry_mode_repo(&skill.source.repo) {
            Some(skill.source.r#ref.clone())
        } else {
            None
        };

        entries.push(LockSkillEntry {
            id: skill.id.clone(),
            source_repo: skill.source.repo.clone(),
            source_subpath: skill.source.subpath.clone(),
            source_ref: skill.source.r#ref.clone(),
            resolved_commit,
            resolved_version,
            install_mode: skill.install.mode.as_str().to_string(),
            installed_at: now.clone(),
            targets,
        });
    }

    Ok(LockFile {
        version: LOCK_VERSION,
        skills: entries,
    })
}

/// Minimal ISO 8601 UTC timestamp formatter (avoids external datetime dependency).
fn utc_now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_epoch_as_iso8601(secs)
}

fn format_epoch_as_iso8601(epoch_secs: u64) -> String {
    let secs = epoch_secs;
    let days = secs / 86400;
    let day_secs = (secs % 86400) as u32;
    let hour = day_secs / 3600;
    let minute = (day_secs % 3600) / 60;
    let second = day_secs % 60;

    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn days_to_ymd(mut days: u64) -> (i32, u32, u32) {
    let mut year = 1970i32;
    loop {
        let diy = if is_leap_year(year) { 366 } else { 365 };
        if days < diy {
            break;
        }
        days -= diy;
        year += 1;
    }
    let month_lengths: [u64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u32;
    for &ml in &month_lengths {
        if days < ml {
            break;
        }
        days -= ml;
        month += 1;
    }
    (year, month, days as u32 + 1)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
