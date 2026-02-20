use std::fs;

use eden_skills_core::agents::detect_installed_agent_targets_from_home;
use eden_skills_core::config::AgentKind;
use tempfile::tempdir;

#[test]
fn detects_all_documented_agent_directories() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path();
    fs::create_dir_all(home.join(".claude")).expect("create .claude");
    fs::create_dir_all(home.join(".cursor")).expect("create .cursor");
    fs::create_dir_all(home.join(".codex")).expect("create .codex");
    fs::create_dir_all(home.join(".codeium/windsurf")).expect("create windsurf");

    let detected = detect_installed_agent_targets_from_home(home);
    assert_eq!(detected.len(), 4);

    assert!(detected
        .iter()
        .any(|target| target.agent == AgentKind::ClaudeCode));
    assert!(detected
        .iter()
        .any(|target| target.agent == AgentKind::Cursor));
    assert!(detected.iter().any(|target| {
        target.agent == AgentKind::Custom
            && target.path.as_deref() == Some("~/.codex/skills")
            && target.environment == "local"
    }));
    assert!(detected.iter().any(|target| {
        target.agent == AgentKind::Custom
            && target.path.as_deref() == Some("~/.codeium/windsurf/skills")
            && target.environment == "local"
    }));
}

#[test]
fn returns_empty_when_no_known_agent_dirs_exist() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path();

    let detected = detect_installed_agent_targets_from_home(home);
    assert!(detected.is_empty());
}
