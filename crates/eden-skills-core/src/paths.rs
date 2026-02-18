use std::env;
use std::path::{Component, Path, PathBuf};

use crate::config::{AgentKind, TargetConfig};
use crate::error::EdenError;

pub fn default_agent_path(agent: &AgentKind) -> Option<&'static str> {
    match agent {
        AgentKind::ClaudeCode => Some("~/.claude/skills"),
        AgentKind::Cursor => Some("~/.cursor/skills"),
        AgentKind::Custom => None,
    }
}

pub fn resolve_target_path(target: &TargetConfig, config_dir: &Path) -> Result<PathBuf, EdenError> {
    if let Some(path) = &target.path {
        return resolve_path_string(path, config_dir);
    }
    if let Some(expected_path) = &target.expected_path {
        return resolve_path_string(expected_path, config_dir);
    }
    let Some(default_path) = default_agent_path(&target.agent) else {
        return Err(EdenError::Validation(
            "TARGET_PATH_UNRESOLVED: no path, expected_path, or default agent path".to_string(),
        ));
    };
    resolve_path_string(default_path, config_dir)
}

pub fn resolve_path_string(input: &str, config_dir: &Path) -> Result<PathBuf, EdenError> {
    if input.trim().is_empty() {
        return Err(EdenError::Validation("path must not be empty".to_string()));
    }

    let expanded = expand_tilde(input)?;
    let resolved = if expanded.is_absolute() {
        expanded
    } else {
        config_dir.join(expanded)
    };
    Ok(normalize_lexical(&resolved))
}

fn expand_tilde(input: &str) -> Result<PathBuf, EdenError> {
    if input == "~" {
        return user_home_dir();
    }
    if let Some(rest) = input.strip_prefix("~/") {
        return Ok(user_home_dir()?.join(rest));
    }
    if input.starts_with('~') {
        return Err(EdenError::Validation(format!(
            "unsupported home expansion in path `{input}`"
        )));
    }
    Ok(PathBuf::from(input))
}

fn user_home_dir() -> Result<PathBuf, EdenError> {
    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(home));
    }
    if let Ok(userprofile) = env::var("USERPROFILE") {
        return Ok(PathBuf::from(userprofile));
    }
    Err(EdenError::Validation(
        "HOME or USERPROFILE is not set for path expansion".to_string(),
    ))
}

pub fn normalize_lexical(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }

    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}
