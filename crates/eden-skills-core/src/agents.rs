use std::env;
use std::path::{Path, PathBuf};

use crate::config::{AgentKind, TargetConfig};
use crate::error::EdenError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AgentDetectionRule {
    detection_subpath: &'static str,
    target: AgentTargetTemplate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentTargetTemplate {
    ClaudeCode,
    Cursor,
    Custom(&'static str),
}

const AGENT_RULES: &[AgentDetectionRule] = &[
    AgentDetectionRule {
        detection_subpath: ".claude",
        target: AgentTargetTemplate::ClaudeCode,
    },
    AgentDetectionRule {
        detection_subpath: ".cursor",
        target: AgentTargetTemplate::Cursor,
    },
    AgentDetectionRule {
        detection_subpath: ".codex",
        target: AgentTargetTemplate::Custom("~/.codex/skills"),
    },
    AgentDetectionRule {
        detection_subpath: ".codeium/windsurf",
        target: AgentTargetTemplate::Custom("~/.codeium/windsurf/skills"),
    },
];

pub fn detect_installed_agent_targets() -> Result<Vec<TargetConfig>, EdenError> {
    let home = user_home_dir()?;
    Ok(detect_installed_agent_targets_from_home(&home))
}

pub fn detect_installed_agent_targets_from_home(home: &Path) -> Vec<TargetConfig> {
    let mut detected = Vec::new();
    for rule in AGENT_RULES {
        if home.join(rule.detection_subpath).is_dir() {
            detected.push(match rule.target {
                AgentTargetTemplate::ClaudeCode => TargetConfig {
                    agent: AgentKind::ClaudeCode,
                    expected_path: None,
                    path: None,
                    environment: "local".to_string(),
                },
                AgentTargetTemplate::Cursor => TargetConfig {
                    agent: AgentKind::Cursor,
                    expected_path: None,
                    path: None,
                    environment: "local".to_string(),
                },
                AgentTargetTemplate::Custom(path) => TargetConfig {
                    agent: AgentKind::Custom,
                    expected_path: None,
                    path: Some(path.to_string()),
                    environment: "local".to_string(),
                },
            });
        }
    }
    detected
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
