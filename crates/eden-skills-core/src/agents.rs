use std::env;
use std::path::{Path, PathBuf};

use crate::config::{AgentKind, TargetConfig};
use crate::error::EdenError;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentDetectionRule {
    detection_subpath: &'static str,
    agent: AgentKind,
}

const AGENT_RULES: &[AgentDetectionRule] = &[
    AgentDetectionRule {
        detection_subpath: ".claude",
        agent: AgentKind::ClaudeCode,
    },
    AgentDetectionRule {
        detection_subpath: ".agents",
        agent: AgentKind::Cursor,
    },
    AgentDetectionRule {
        detection_subpath: ".agent",
        agent: AgentKind::Antigravity,
    },
    AgentDetectionRule {
        detection_subpath: ".augment",
        agent: AgentKind::Augment,
    },
    AgentDetectionRule {
        detection_subpath: "skills",
        agent: AgentKind::Openclaw,
    },
    AgentDetectionRule {
        detection_subpath: ".cline",
        agent: AgentKind::Cline,
    },
    AgentDetectionRule {
        detection_subpath: ".codebuddy",
        agent: AgentKind::Codebuddy,
    },
    AgentDetectionRule {
        detection_subpath: ".commandcode",
        agent: AgentKind::CommandCode,
    },
    AgentDetectionRule {
        detection_subpath: ".continue",
        agent: AgentKind::Continue,
    },
    AgentDetectionRule {
        detection_subpath: ".cortex",
        agent: AgentKind::Cortex,
    },
    AgentDetectionRule {
        detection_subpath: ".crush",
        agent: AgentKind::Crush,
    },
    AgentDetectionRule {
        detection_subpath: ".factory",
        agent: AgentKind::Droid,
    },
    AgentDetectionRule {
        detection_subpath: ".goose",
        agent: AgentKind::Goose,
    },
    AgentDetectionRule {
        detection_subpath: ".junie",
        agent: AgentKind::Junie,
    },
    AgentDetectionRule {
        detection_subpath: ".iflow",
        agent: AgentKind::IflowCli,
    },
    AgentDetectionRule {
        detection_subpath: ".kilocode",
        agent: AgentKind::Kilo,
    },
    AgentDetectionRule {
        detection_subpath: ".kiro",
        agent: AgentKind::KiroCli,
    },
    AgentDetectionRule {
        detection_subpath: ".kode",
        agent: AgentKind::Kode,
    },
    AgentDetectionRule {
        detection_subpath: ".mcpjam",
        agent: AgentKind::Mcpjam,
    },
    AgentDetectionRule {
        detection_subpath: ".vibe",
        agent: AgentKind::MistralVibe,
    },
    AgentDetectionRule {
        detection_subpath: ".mux",
        agent: AgentKind::Mux,
    },
    AgentDetectionRule {
        detection_subpath: ".openhands",
        agent: AgentKind::Openhands,
    },
    AgentDetectionRule {
        detection_subpath: ".pi",
        agent: AgentKind::Pi,
    },
    AgentDetectionRule {
        detection_subpath: ".qoder",
        agent: AgentKind::Qoder,
    },
    AgentDetectionRule {
        detection_subpath: ".qwen",
        agent: AgentKind::QwenCode,
    },
    AgentDetectionRule {
        detection_subpath: ".roo",
        agent: AgentKind::Roo,
    },
    AgentDetectionRule {
        detection_subpath: ".trae",
        agent: AgentKind::Trae,
    },
    AgentDetectionRule {
        detection_subpath: ".windsurf",
        agent: AgentKind::Windsurf,
    },
    AgentDetectionRule {
        detection_subpath: ".zencoder",
        agent: AgentKind::Zencoder,
    },
    AgentDetectionRule {
        detection_subpath: ".neovate",
        agent: AgentKind::Neovate,
    },
    AgentDetectionRule {
        detection_subpath: ".pochi",
        agent: AgentKind::Pochi,
    },
    AgentDetectionRule {
        detection_subpath: ".adal",
        agent: AgentKind::Adal,
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
