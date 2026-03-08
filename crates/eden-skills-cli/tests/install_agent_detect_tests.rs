mod common;

use std::fs;
use std::path::Path;

use tempfile::tempdir;

#[test]
fn install_without_target_detects_multiple_agent_directories() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(home_dir.join(".claude/skills")).expect("create .claude/skills");
    fs::create_dir_all(home_dir.join(".cursor/skills")).expect("create .cursor/skills");
    let repo_dir = temp.path().join("agent-detect-repo");
    write_root_skill_repo(&repo_dir, "agent-skill");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
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
    let output = common::eden_command(&home_dir)
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
fn install_without_target_detects_parent_only_global_agent_root() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(home_dir.join(".config/opencode")).expect("create .config/opencode");
    let repo_dir = temp.path().join("parent-only-opencode-repo");
    write_root_skill_repo(&repo_dir, "opencode-parent-skill");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./parent-only-opencode-repo", "--config"])
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
        home_dir
            .join(".config/opencode/skills/opencode-parent-skill")
            .exists(),
        "opencode target should be installed even when only parent dir existed initially"
    );

    let agents = read_first_skill_target_agents(&config_path);
    assert_eq!(agents, vec!["opencode".to_string()]);
}

#[test]
fn repeated_install_backfills_newly_detected_agent_target_for_existing_skill() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(home_dir.join(".claude/skills")).expect("create .claude/skills");
    let repo_dir = temp.path().join("reinstall-backfill-repo");
    write_root_skill_repo(&repo_dir, "backfill-skill");

    let config_path = temp.path().join("skills.toml");
    let first = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./reinstall-backfill-repo", "--config"])
        .arg(&config_path)
        .output()
        .expect("run first install");
    assert_eq!(
        first.status.code(),
        Some(0),
        "first install should succeed, stderr={}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        home_dir.join(".claude/skills/backfill-skill").exists(),
        "first install should install to initial detected claude target"
    );
    assert_eq!(
        read_first_skill_target_agents(&config_path),
        vec!["claude-code".to_string()],
        "first install should persist only initial detected agent"
    );

    fs::create_dir_all(home_dir.join(".config/opencode")).expect("create .config/opencode");

    let second = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./reinstall-backfill-repo", "--config"])
        .arg(&config_path)
        .output()
        .expect("run second install");
    assert_eq!(
        second.status.code(),
        Some(0),
        "second install should succeed, stderr={}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert!(
        home_dir.join(".claude/skills/backfill-skill").exists(),
        "second install should keep existing claude target installed"
    );
    assert!(
        home_dir
            .join(".config/opencode/skills/backfill-skill")
            .exists(),
        "second install should backfill newly detected opencode target"
    );

    let agents = read_first_skill_target_agents(&config_path);
    assert_eq!(
        agents.len(),
        2,
        "targets should be replaced by detected set"
    );
    assert!(
        agents.iter().any(|agent| agent == "claude-code"),
        "targets should include existing agent"
    );
    assert!(
        agents.iter().any(|agent| agent == "opencode"),
        "targets should include newly detected agent"
    );
}

#[test]
fn explicit_target_override_skips_auto_detection() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(home_dir.join(".claude/skills")).expect("create .claude/skills");
    fs::create_dir_all(home_dir.join(".cursor/skills")).expect("create .cursor/skills");
    let repo_dir = temp.path().join("override-repo");
    write_root_skill_repo(&repo_dir, "override-skill");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
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

#[test]
fn explicit_shared_global_target_alias_installs_to_config_agents_path() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("shared-global-target-repo");
    write_root_skill_repo(&repo_dir, "shared-global-skill");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./shared-global-target-repo",
            "--target",
            "kimi-cli",
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
        home_dir
            .join(".config/agents/skills/shared-global-skill")
            .exists(),
        "kimi-cli should install into ~/.config/agents/skills"
    );

    let agents = read_first_skill_target_agents(&config_path);
    assert_eq!(agents, vec!["kimi-cli".to_string()]);
}

#[cfg(unix)]
#[test]
fn install_with_docker_target_detects_multiple_agents_inside_container() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let path_dir = temp.path().join("docker-stub-bin");
    let state_dir = temp.path().join("docker-state");
    fs::create_dir_all(&path_dir).expect("create path dir");
    fs::create_dir_all(&state_dir).expect("create docker state dir");
    write_docker_stub(
        &path_dir.join("docker"),
        &state_dir,
        "my-container",
        &[".claude", ".cursor"],
        "[]",
    );

    let repo_dir = temp.path().join("docker-agent-detect-repo");
    write_root_skill_repo(&repo_dir, "docker-agent-skill");
    let config_path = temp.path().join("skills.toml");

    let output = common::eden_command(&home_dir)
        .env("PATH", &path_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./docker-agent-detect-repo",
            "--target",
            "docker:my-container",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run docker-target install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "docker-target install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let targets = read_first_skill_targets(&config_path);
    assert_eq!(targets.len(), 2, "expected one target per detected agent");
    assert!(
        targets.iter().any(|target| {
            target.agent == "claude-code" && target.environment == "docker:my-container"
        }),
        "expected claude-code docker target, targets={targets:?}"
    );
    assert!(
        targets
            .iter()
            .any(|target| target.agent == "cursor" && target.environment == "docker:my-container"),
        "expected cursor docker target, targets={targets:?}"
    );
}

#[cfg(unix)]
#[test]
fn install_with_docker_target_falls_back_to_claude_when_container_has_no_agents() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let path_dir = temp.path().join("docker-stub-bin");
    let state_dir = temp.path().join("docker-state");
    fs::create_dir_all(&path_dir).expect("create path dir");
    fs::create_dir_all(&state_dir).expect("create docker state dir");
    write_docker_stub(
        &path_dir.join("docker"),
        &state_dir,
        "my-container",
        &[],
        "[]",
    );

    let repo_dir = temp.path().join("docker-agent-fallback-repo");
    write_root_skill_repo(&repo_dir, "docker-fallback-skill");
    let config_path = temp.path().join("skills.toml");

    let output = common::eden_command(&home_dir)
        .env("PATH", &path_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./docker-agent-fallback-repo",
            "--target",
            "docker:my-container",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run docker-target install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "docker-target install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "No installed agents detected in container 'my-container'; defaulting to claude-code."
        ),
        "stderr={stderr}"
    );

    let targets = read_first_skill_targets(&config_path);
    assert_eq!(
        targets,
        vec![InstalledTarget {
            agent: "claude-code".to_string(),
            environment: "docker:my-container".to_string(),
            path: None,
        }]
    );
}

#[cfg(unix)]
#[test]
fn repeated_install_without_docker_target_preserves_existing_manual_docker_targets() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(home_dir.join(".claude/skills")).expect("create local claude skills");

    let path_dir = temp.path().join("docker-stub-bin");
    let state_dir = temp.path().join("docker-state");
    fs::create_dir_all(&path_dir).expect("create path dir");
    fs::create_dir_all(&state_dir).expect("create docker state dir");
    write_docker_stub(
        &path_dir.join("docker"),
        &state_dir,
        "test-container",
        &[],
        "[]",
    );

    let repo_dir = temp.path().join("manual-docker-target-repo");
    write_root_skill_repo(&repo_dir, "manual-docker-skill");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "./storage"

[[skills]]
id = "manual-docker-skill"

[skills.source]
repo = "file://{repo}"
subpath = "."
ref = "main"

[skills.install]
mode = "copy"

[[skills.targets]]
agent = "cursor"
path = "/root/.cursor/skills"
environment = "docker:test-container"

[skills.verify]
enabled = true
checks = ["path-exists", "content-present"]

[skills.safety]
no_exec_metadata_only = false
"#,
            repo = repo_dir.display()
        ),
    )
    .expect("write manual docker config");

    let output = common::eden_command(&home_dir)
        .env("PATH", &path_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./manual-docker-target-repo",
            "--id",
            "manual-docker-skill",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run reinstall");
    assert_eq!(
        output.status.code(),
        Some(0),
        "reinstall should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let targets = read_first_skill_targets(&config_path);
    assert_eq!(
        targets,
        vec![InstalledTarget {
            agent: "cursor".to_string(),
            environment: "docker:test-container".to_string(),
            path: Some("/root/.cursor/skills".to_string()),
        }],
        "manual docker targets should remain unchanged when --target docker:... is not used"
    );
    assert!(
        !home_dir.join(".claude/skills/manual-docker-skill").exists(),
        "local auto-detection must not replace an existing manual docker target"
    );
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

#[cfg(unix)]
fn write_docker_stub(
    docker_bin: &Path,
    state_dir: &Path,
    container_name: &str,
    detected_dirs: &[&str],
    mounts_json: &str,
) {
    let copied_targets = state_dir.join("copied-targets.log");
    let detected = detected_dirs.join("\\n");
    let script = format!(
        r#"#!/bin/sh
set -eu
copied_targets="{copied_targets}"
cmd="$1"
shift
if [ "$cmd" = "--version" ]; then
  echo "Docker version 27.0.0"
  exit 0
fi
if [ "$cmd" = "inspect" ]; then
  if [ "$1" = "--format" ] && [ "$2" = "{{{{.State.Running}}}}" ] && [ "$3" = "{container_name}" ]; then
    echo "true"
    exit 0
  fi
  if [ "$1" = "--format" ] && [ "$2" = "{{{{json .Mounts}}}}" ] && [ "$3" = "{container_name}" ]; then
    printf '%s\n' '{mounts_json}'
    exit 0
  fi
fi
if [ "$cmd" = "cp" ]; then
  dst="$2"
  printf '%s\n' "${{dst#*:}}" >> "$copied_targets"
  exit 0
fi
if [ "$cmd" = "exec" ]; then
  container="$1"
  shift
  if [ "$container" != "{container_name}" ]; then
    echo "container not found" >&2
    exit 1
  fi
  if [ "$1" = "sh" ] && [ "$2" = "-c" ]; then
    case "$3" in
      *for\ d\ in*)
        if [ -n "{detected}" ]; then
          printf '%b\n' "{detected}"
        fi
        exit 0
        ;;
      *HOME*)
        printf '%s' '/root'
        exit 0
        ;;
      test\ -e\ *)
        path="${{3#test -e }}"
        path="${{path#\"}}"
        path="${{path%\"}}"
        if [ -f "$copied_targets" ] && grep -Fxq "$path" "$copied_targets"; then
          exit 0
        fi
        exit 1
        ;;
      rm\ -rf\ *)
        exit 0
        ;;
      *)
        exit 0
        ;;
    esac
  fi
fi
echo "unsupported docker call: $cmd" >&2
exit 1
"#,
        copied_targets = copied_targets.display(),
        container_name = container_name,
        mounts_json = mounts_json,
        detected = detected
    );
    fs::write(docker_bin, script).expect("write docker stub");
    let mut perms = fs::metadata(docker_bin)
        .expect("docker stub metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(docker_bin, perms).expect("set docker stub executable");
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

#[cfg(unix)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct InstalledTarget {
    agent: String,
    environment: String,
    path: Option<String>,
}

#[cfg(unix)]
fn read_first_skill_targets(config_path: &Path) -> Vec<InstalledTarget> {
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
                .map(|target| InstalledTarget {
                    agent: target
                        .get("agent")
                        .and_then(|agent| agent.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    environment: target
                        .get("environment")
                        .and_then(|environment| environment.as_str())
                        .unwrap_or("local")
                        .to_string(),
                    path: target
                        .get("path")
                        .and_then(|path| path.as_str())
                        .map(ToString::to_string),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
