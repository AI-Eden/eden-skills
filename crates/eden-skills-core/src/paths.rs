use std::env;
use std::path::{Component, Path, PathBuf};

use crate::config::{AgentKind, TargetConfig};
use crate::error::EdenError;

const KNOWN_DEFAULT_AGENT_PATHS: &[&str] = &[
    "~/.adal/skills",
    "~/.agent/skills",
    "~/.agents/skills",
    "~/.augment/skills",
    "~/.claude/skills",
    "~/.cline/skills",
    "~/.codebuddy/skills",
    "~/.commandcode/skills",
    "~/.continue/skills",
    "~/.cortex/skills",
    "~/.crush/skills",
    "~/.factory/skills",
    "~/.goose/skills",
    "~/.iflow/skills",
    "~/.junie/skills",
    "~/.kilocode/skills",
    "~/.kiro/skills",
    "~/.kode/skills",
    "~/.mcpjam/skills",
    "~/.mux/skills",
    "~/.neovate/skills",
    "~/.openhands/skills",
    "~/.pi/skills",
    "~/.pochi/skills",
    "~/.qoder/skills",
    "~/.qwen/skills",
    "~/.roo/skills",
    "~/.trae/skills",
    "~/.vibe/skills",
    "~/.windsurf/skills",
    "~/.zencoder/skills",
    "~/skills",
];

pub fn default_agent_path(agent: &AgentKind) -> Option<&'static str> {
    match agent {
        AgentKind::Amp => Some("~/.agents/skills"),
        AgentKind::Adal => Some("~/.adal/skills"),
        AgentKind::Antigravity => Some("~/.agent/skills"),
        AgentKind::Augment => Some("~/.augment/skills"),
        AgentKind::ClaudeCode => Some("~/.claude/skills"),
        AgentKind::Cline => Some("~/.cline/skills"),
        AgentKind::Codebuddy => Some("~/.codebuddy/skills"),
        AgentKind::Codex => Some("~/.agents/skills"),
        AgentKind::CommandCode => Some("~/.commandcode/skills"),
        AgentKind::Continue => Some("~/.continue/skills"),
        AgentKind::Cortex => Some("~/.cortex/skills"),
        AgentKind::Crush => Some("~/.crush/skills"),
        AgentKind::Cursor => Some("~/.agents/skills"),
        AgentKind::Droid => Some("~/.factory/skills"),
        AgentKind::GeminiCli => Some("~/.agents/skills"),
        AgentKind::GithubCopilot => Some("~/.agents/skills"),
        AgentKind::Goose => Some("~/.goose/skills"),
        AgentKind::IflowCli => Some("~/.iflow/skills"),
        AgentKind::Junie => Some("~/.junie/skills"),
        AgentKind::Kilo => Some("~/.kilocode/skills"),
        AgentKind::KimiCli => Some("~/.agents/skills"),
        AgentKind::KiroCli => Some("~/.kiro/skills"),
        AgentKind::Kode => Some("~/.kode/skills"),
        AgentKind::Mcpjam => Some("~/.mcpjam/skills"),
        AgentKind::MistralVibe => Some("~/.vibe/skills"),
        AgentKind::Mux => Some("~/.mux/skills"),
        AgentKind::Neovate => Some("~/.neovate/skills"),
        AgentKind::Openclaw => Some("~/skills"),
        AgentKind::Opencode => Some("~/.agents/skills"),
        AgentKind::Openhands => Some("~/.openhands/skills"),
        AgentKind::Pi => Some("~/.pi/skills"),
        AgentKind::Pochi => Some("~/.pochi/skills"),
        AgentKind::Qoder => Some("~/.qoder/skills"),
        AgentKind::QwenCode => Some("~/.qwen/skills"),
        AgentKind::Replit => Some("~/.agents/skills"),
        AgentKind::Roo => Some("~/.roo/skills"),
        AgentKind::Trae => Some("~/.trae/skills"),
        AgentKind::TraeCn => Some("~/.trae/skills"),
        AgentKind::Universal => Some("~/.agents/skills"),
        AgentKind::Windsurf => Some("~/.windsurf/skills"),
        AgentKind::Zencoder => Some("~/.zencoder/skills"),
        AgentKind::Custom => None,
    }
}

pub fn known_default_agent_paths() -> &'static [&'static str] {
    KNOWN_DEFAULT_AGENT_PATHS
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
