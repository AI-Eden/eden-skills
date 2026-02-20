use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn install_without_target_detects_multiple_agent_directories() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(home_dir.join(".claude")).expect("create .claude");
    fs::create_dir_all(home_dir.join(".cursor")).expect("create .cursor");
    let repo_dir = temp.path().join("agent-detect-repo");
    write_root_skill_repo(&repo_dir, "agent-skill");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./agent-detect-repo", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        home_dir.join(".claude/skills/agent-skill").exists(),
        "claude target should be installed"
    );
    assert!(
        home_dir.join(".cursor/skills/agent-skill").exists(),
        "cursor target should be installed"
    );

    let agents = read_first_skill_target_agents(&config_path);
    assert_eq!(
        agents,
        vec!["claude-code".to_string(), "cursor".to_string()]
    );
}

#[test]
fn install_without_target_falls_back_to_claude_with_warning() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("fallback-repo");
    write_root_skill_repo(&repo_dir, "fallback-skill");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./fallback-repo", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "No installed agents detected; defaulting to claude-code (~/.claude/skills/)"
        ),
        "stderr={stderr}"
    );
    assert!(
        home_dir.join(".claude/skills/fallback-skill").exists(),
        "fallback claude target should be installed"
    );

    let agents = read_first_skill_target_agents(&config_path);
    assert_eq!(agents, vec!["claude-code".to_string()]);
}

#[test]
fn explicit_target_override_skips_auto_detection() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(home_dir.join(".claude")).expect("create .claude");
    fs::create_dir_all(home_dir.join(".cursor")).expect("create .cursor");
    let repo_dir = temp.path().join("override-repo");
    write_root_skill_repo(&repo_dir, "override-skill");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./override-repo",
            "--target",
            "cursor",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        home_dir.join(".cursor/skills/override-skill").exists(),
        "cursor target should be installed"
    );
    assert!(
        !home_dir.join(".claude/skills/override-skill").exists(),
        "claude target should not be installed when --target cursor is provided"
    );

    let agents = read_first_skill_target_agents(&config_path);
    assert_eq!(agents, vec!["cursor".to_string()]);
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

fn write_root_skill_repo(repo_dir: &Path, name: &str) {
    fs::create_dir_all(repo_dir).expect("create repo dir");
    fs::write(
        repo_dir.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: test\n---\n"),
    )
    .expect("write SKILL.md");
    fs::write(repo_dir.join("README.md"), "demo").expect("write README");
}

fn read_first_skill_target_agents(config_path: &Path) -> Vec<String> {
    let config_text = fs::read_to_string(config_path).expect("read config");
    let value: toml::Value = toml::from_str(&config_text).expect("valid config toml");
    value
        .get("skills")
        .and_then(|skills| skills.as_array())
        .and_then(|skills| skills.first())
        .and_then(|skill| skill.get("targets"))
        .and_then(|targets| targets.as_array())
        .map(|targets| {
            targets
                .iter()
                .filter_map(|target| target.get("agent").and_then(|agent| agent.as_str()))
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
