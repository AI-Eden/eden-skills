use std::fs;

use eden_skills_core::agents::detect_installed_agent_targets_from_home;
use eden_skills_core::config::AgentKind;
use tempfile::tempdir;

#[test]
fn detects_all_documented_agent_directories() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path();
    fs::create_dir_all(home.join(".claude/skills")).expect("create .claude/skills");
    fs::create_dir_all(home.join(".cursor/skills")).expect("create .cursor/skills");
    fs::create_dir_all(home.join(".codex/skills")).expect("create .codex/skills");
    fs::create_dir_all(home.join(".codeium/windsurf/skills"))
        .expect("create .codeium/windsurf/skills");
    fs::create_dir_all(home.join(".adal/skills")).expect("create .adal/skills");

    let detected = detect_installed_agent_targets_from_home(home);
    assert_eq!(detected.len(), 5);

    assert!(detected
        .iter()
        .any(|target| target.agent == AgentKind::ClaudeCode));
    assert!(detected
        .iter()
        .any(|target| target.agent == AgentKind::Cursor));
    assert!(detected
        .iter()
        .any(|target| target.agent == AgentKind::Codex));
    assert!(detected
        .iter()
        .any(|target| target.agent == AgentKind::Windsurf));
    assert!(detected
        .iter()
        .any(|target| target.agent == AgentKind::Adal));
}

#[test]
fn returns_empty_when_no_known_agent_dirs_exist() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path();

    let detected = detect_installed_agent_targets_from_home(home);
    assert!(detected.is_empty());
}

#[test]
fn detects_global_agent_skill_directories() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path();
    fs::create_dir_all(home.join(".cursor/skills")).expect("create .cursor/skills");
    fs::create_dir_all(home.join(".codeium/windsurf/skills"))
        .expect("create .codeium/windsurf/skills");

    let detected = detect_installed_agent_targets_from_home(home);

    assert!(
        detected.iter().any(|target| {
            target.agent == AgentKind::Cursor
                && target.path.is_none()
                && target.environment == "local"
        }),
        "expected .cursor/skills to be detected as cursor target"
    );
    assert!(
        detected.iter().any(|target| {
            target.agent == AgentKind::Windsurf
                && target.path.is_none()
                && target.environment == "local"
        }),
        "expected .codeium/windsurf/skills to be detected as windsurf target"
    );
}

#[test]
fn does_not_auto_detect_shared_config_agents_path() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path();
    fs::create_dir_all(home.join(".config/agents/skills")).expect("create .config/agents/skills");

    let detected = detect_installed_agent_targets_from_home(home);
    assert!(
        !detected.iter().any(|target| target.agent == AgentKind::Amp
            || target.agent == AgentKind::KimiCli
            || target.agent == AgentKind::Replit
            || target.agent == AgentKind::Universal),
        "shared ~/.config/agents/skills should not create ambiguous auto-detection targets"
    );
}
