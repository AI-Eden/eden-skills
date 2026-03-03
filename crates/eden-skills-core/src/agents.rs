use std::env;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::config::{AgentKind, TargetConfig};
use crate::error::EdenError;
use crate::paths::default_agent_path;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentDetectionRule {
    detection_subpath: &'static str,
    agent: AgentKind,
}

static AGENT_RULES: OnceLock<Vec<AgentDetectionRule>> = OnceLock::new();

fn agent_rules() -> &'static [AgentDetectionRule] {
    AGENT_RULES
        .get_or_init(|| {
            AgentKind::all_non_custom()
                .iter()
                .filter(|agent| agent.is_auto_detect_eligible())
                .filter_map(|agent| {
                    let default_path = default_agent_path(agent)?;
                    let detection_subpath = default_path.strip_prefix("~/")?;
                    Some(AgentDetectionRule {
                        detection_subpath,
                        agent: agent.clone(),
                    })
                })
                .collect()
        })
        .as_slice()
}

pub fn detect_installed_agent_targets() -> Result<Vec<TargetConfig>, EdenError> {
    let home = user_home_dir()?;
    Ok(detect_installed_agent_targets_from_home(&home))
}

pub fn detect_installed_agent_targets_from_home(home: &Path) -> Vec<TargetConfig> {
    let mut detected = Vec::new();
    for rule in agent_rules() {
        if home.join(rule.detection_subpath).is_dir() {
            detected.push(TargetConfig {
                agent: rule.agent.clone(),
                expected_path: None,
                path: None,
                environment: "local".to_string(),
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
