//! Path resolution, tilde expansion, and agent default-path registry.
//!
//! Provides the canonical mapping from [`AgentKind`] to its default
//! global and project-scope skill directories, tilde (`~`) expansion,
//! lexical path normalization (no filesystem access), and target-path
//! resolution for both local and Docker environments.

use std::env;
use std::path::{Component, Path, PathBuf};

use crate::config::{AgentKind, TargetConfig};
use crate::error::EdenError;

/// Well-known global skill directories across all supported agents.
/// Used by auto-detection to probe the filesystem for installed agents.
const KNOWN_DEFAULT_AGENT_PATHS: &[&str] = &[
    "~/.adal/skills",
    "~/.agents/skills",
    "~/.augment/skills",
    "~/.claude/skills",
    "~/.codeium/windsurf/skills",
    "~/.codex/skills",
    "~/.codebuddy/skills",
    "~/.commandcode/skills",
    "~/.config/agents/skills",
    "~/.config/crush/skills",
    "~/.config/goose/skills",
    "~/.config/opencode/skills",
    "~/.continue/skills",
    "~/.copilot/skills",
    "~/.cursor/skills",
    "~/.factory/skills",
    "~/.gemini/antigravity/skills",
    "~/.gemini/skills",
    "~/.iflow/skills",
    "~/.junie/skills",
    "~/.kilocode/skills",
    "~/.kiro/skills",
    "~/.kode/skills",
    "~/.mcpjam/skills",
    "~/.mux/skills",
    "~/.neovate/skills",
    "~/.openhands/skills",
    "~/.openclaw/skills",
    "~/.pi/agent/skills",
    "~/.pochi/skills",
    "~/.qoder/skills",
    "~/.qwen/skills",
    "~/.roo/skills",
    "~/.snowflake/cortex/skills",
    "~/.trae-cn/skills",
    "~/.trae/skills",
    "~/.vibe/skills",
    "~/.zencoder/skills",
];

/// Return the default global skill directory for the given agent, or
/// `None` for [`AgentKind::Custom`].
pub fn default_agent_path(agent: &AgentKind) -> Option<&'static str> {
    match agent {
        AgentKind::Amp => Some("~/.config/agents/skills"),
        AgentKind::Adal => Some("~/.adal/skills"),
        AgentKind::Antigravity => Some("~/.gemini/antigravity/skills"),
        AgentKind::Augment => Some("~/.augment/skills"),
        AgentKind::ClaudeCode => Some("~/.claude/skills"),
        AgentKind::Cline => Some("~/.agents/skills"),
        AgentKind::Codebuddy => Some("~/.codebuddy/skills"),
        AgentKind::Codex => Some("~/.codex/skills"),
        AgentKind::CommandCode => Some("~/.commandcode/skills"),
        AgentKind::Continue => Some("~/.continue/skills"),
        AgentKind::Cortex => Some("~/.snowflake/cortex/skills"),
        AgentKind::Crush => Some("~/.config/crush/skills"),
        AgentKind::Cursor => Some("~/.cursor/skills"),
        AgentKind::Droid => Some("~/.factory/skills"),
        AgentKind::GeminiCli => Some("~/.gemini/skills"),
        AgentKind::GithubCopilot => Some("~/.copilot/skills"),
        AgentKind::Goose => Some("~/.config/goose/skills"),
        AgentKind::IflowCli => Some("~/.iflow/skills"),
        AgentKind::Junie => Some("~/.junie/skills"),
        AgentKind::Kilo => Some("~/.kilocode/skills"),
        AgentKind::KimiCli => Some("~/.config/agents/skills"),
        AgentKind::KiroCli => Some("~/.kiro/skills"),
        AgentKind::Kode => Some("~/.kode/skills"),
        AgentKind::Mcpjam => Some("~/.mcpjam/skills"),
        AgentKind::MistralVibe => Some("~/.vibe/skills"),
        AgentKind::Mux => Some("~/.mux/skills"),
        AgentKind::Neovate => Some("~/.neovate/skills"),
        AgentKind::Openclaw => Some("~/.openclaw/skills"),
        AgentKind::Opencode => Some("~/.config/opencode/skills"),
        AgentKind::Openhands => Some("~/.openhands/skills"),
        AgentKind::Pi => Some("~/.pi/agent/skills"),
        AgentKind::Pochi => Some("~/.pochi/skills"),
        AgentKind::Qoder => Some("~/.qoder/skills"),
        AgentKind::QwenCode => Some("~/.qwen/skills"),
        AgentKind::Replit => Some("~/.config/agents/skills"),
        AgentKind::Roo => Some("~/.roo/skills"),
        AgentKind::Trae => Some("~/.trae/skills"),
        AgentKind::TraeCn => Some("~/.trae-cn/skills"),
        AgentKind::Universal => Some("~/.config/agents/skills"),
        AgentKind::Windsurf => Some("~/.codeium/windsurf/skills"),
        AgentKind::Zencoder => Some("~/.zencoder/skills"),
        AgentKind::Custom => None,
    }
}

/// Project-scope skill root for each supported agent, aligned with
/// vercel-labs/skills "Supported Agents" -> "Project Path".
///
/// This is intentionally separate from `default_agent_path` (global scope)
/// because project discovery and global installation have different semantics.
pub fn default_agent_project_path(agent: &AgentKind) -> Option<&'static str> {
    match agent {
        AgentKind::Amp => Some(".agents/skills"),
        AgentKind::Adal => Some(".adal/skills"),
        AgentKind::Antigravity => Some(".agent/skills"),
        AgentKind::Augment => Some(".augment/skills"),
        AgentKind::ClaudeCode => Some(".claude/skills"),
        AgentKind::Cline => Some(".agents/skills"),
        AgentKind::Codebuddy => Some(".codebuddy/skills"),
        AgentKind::Codex => Some(".agents/skills"),
        AgentKind::CommandCode => Some(".commandcode/skills"),
        AgentKind::Continue => Some(".continue/skills"),
        AgentKind::Cortex => Some(".cortex/skills"),
        AgentKind::Crush => Some(".crush/skills"),
        AgentKind::Cursor => Some(".agents/skills"),
        AgentKind::Droid => Some(".factory/skills"),
        AgentKind::GeminiCli => Some(".agents/skills"),
        AgentKind::GithubCopilot => Some(".agents/skills"),
        AgentKind::Goose => Some(".goose/skills"),
        AgentKind::IflowCli => Some(".iflow/skills"),
        AgentKind::Junie => Some(".junie/skills"),
        AgentKind::Kilo => Some(".kilocode/skills"),
        AgentKind::KimiCli => Some(".agents/skills"),
        AgentKind::KiroCli => Some(".kiro/skills"),
        AgentKind::Kode => Some(".kode/skills"),
        AgentKind::Mcpjam => Some(".mcpjam/skills"),
        AgentKind::MistralVibe => Some(".vibe/skills"),
        AgentKind::Mux => Some(".mux/skills"),
        AgentKind::Neovate => Some(".neovate/skills"),
        AgentKind::Openclaw => Some("skills"),
        AgentKind::Opencode => Some(".agents/skills"),
        AgentKind::Openhands => Some(".openhands/skills"),
        AgentKind::Pi => Some(".pi/skills"),
        AgentKind::Pochi => Some(".pochi/skills"),
        AgentKind::Qoder => Some(".qoder/skills"),
        AgentKind::QwenCode => Some(".qwen/skills"),
        AgentKind::Replit => Some(".agents/skills"),
        AgentKind::Roo => Some(".roo/skills"),
        AgentKind::Trae => Some(".trae/skills"),
        AgentKind::TraeCn => Some(".trae/skills"),
        AgentKind::Universal => Some(".agents/skills"),
        AgentKind::Windsurf => Some(".windsurf/skills"),
        AgentKind::Zencoder => Some(".zencoder/skills"),
        AgentKind::Custom => None,
    }
}

/// Expose the full list of known default agent paths for filesystem probing.
pub fn known_default_agent_paths() -> &'static [&'static str] {
    KNOWN_DEFAULT_AGENT_PATHS
}

/// Returns all `AgentKind` values that share the same default install path as
/// `primary`, with `primary` listed first. Derived entirely from
/// `default_agent_path` — no extra data source.
pub fn colocated_agents(primary: &AgentKind) -> Vec<AgentKind> {
    let Some(target_path) = default_agent_path(primary) else {
        return vec![primary.clone()];
    };
    let mut result = vec![primary.clone()];
    for agent in AgentKind::all_non_custom() {
        if agent != primary && default_agent_path(agent) == Some(target_path) {
            result.push(agent.clone());
        }
    }
    result
}

/// Returns a slash-separated display label that includes all agents sharing
/// the same default install path as `primary`. E.g. for `amp`:
/// `"amp/kimi-cli/replit/universal"`.
/// Falls back to the plain agent name when the path is unique.
pub fn colocated_agent_display_label(primary: &AgentKind) -> String {
    let agents = colocated_agents(primary);
    agents
        .iter()
        .map(AgentKind::as_str)
        .collect::<Vec<_>>()
        .join("/")
}

/// Resolve the absolute install target path for a skill target entry,
/// trying `path` → `expected_path` → `default_agent_path` in order.
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

/// Expand `~` and resolve relative paths against `config_dir`, then
/// normalize the result lexically (without touching the filesystem).
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

/// Pure lexical normalization: collapse `.` and `..` components without
/// any filesystem access.  Returns `"."` for an empty result.
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
