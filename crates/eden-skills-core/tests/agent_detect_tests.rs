use std::fs;

use eden_skills_core::agents::detect_installed_agent_targets_from_home;
use eden_skills_core::config::AgentKind;
use tempfile::tempdir;

#[test]
fn detects_all_documented_agent_directories() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path();
    fs::create_dir_all(home.join(".claude")).expect("create .claude");
    fs::create_dir_all(home.join(".agents")).expect("create .agents");
    fs::create_dir_all(home.join(".windsurf")).expect("create .windsurf");
    fs::create_dir_all(home.join(".adal")).expect("create .adal");

    let detected = detect_installed_agent_targets_from_home(home);
    assert_eq!(detected.len(), 4);

    assert!(detected
        .iter()
        .any(|target| target.agent == AgentKind::ClaudeCode));
    assert!(detected
        .iter()
        .any(|target| target.agent == AgentKind::Cursor));
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
fn detects_project_path_derived_global_agent_directories() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path();
    fs::create_dir_all(home.join(".agents/skills")).expect("create .agents/skills");
    fs::create_dir_all(home.join(".windsurf/skills")).expect("create .windsurf/skills");

    let detected = detect_installed_agent_targets_from_home(home);

    assert!(
        detected.iter().any(|target| {
            target.agent == AgentKind::Cursor
                && target.path.is_none()
                && target.environment == "local"
        }),
        "expected .agents/skills to be detected as a built-in agent target"
    );
    assert!(
        detected.iter().any(|target| {
            target.agent == AgentKind::Windsurf
                && target.path.is_none()
                && target.environment == "local"
        }),
        "expected .windsurf/skills to be detected as windsurf target"
    );
}
