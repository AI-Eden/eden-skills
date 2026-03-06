#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
use tempfile::tempdir;

#[cfg(unix)]
#[test]
fn docker_mount_hint_outputs_recommended_bind_mounts() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let storage_root = temp.path().join("storage");
    let path_dir = temp.path().join("docker-stub-bin");
    fs::create_dir_all(home_dir.join(".claude/skills")).expect("create host claude skills");
    fs::create_dir_all(home_dir.join(".cursor/skills")).expect("create host cursor skills");
    fs::create_dir_all(&path_dir).expect("create docker stub dir");

    write_docker_stub(
        &path_dir.join("docker"),
        temp.path(),
        "my-container",
        "/root",
        &[],
        "[]",
    );
    let config_path = write_mount_hint_config(temp.path(), &storage_root, "my-container");

    let output = eden_command(&home_dir)
        .env("PATH", &path_dir)
        .args(["docker", "mount-hint", "my-container", "--config"])
        .arg(&config_path)
        .output()
        .expect("run docker mount-hint");
    assert_eq!(
        output.status.code(),
        Some(0),
        "docker mount-hint should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Docker mount-hint for container 'my-container'"),
        "stdout={stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "-v {}:/root/.eden-skills/skills:ro",
            storage_root.display()
        )),
        "stdout={stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "-v {}:/root/.claude/skills",
            home_dir.join(".claude/skills").display()
        )),
        "stdout={stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "-v {}:/root/.cursor/skills",
            home_dir.join(".cursor/skills").display()
        )),
        "stdout={stdout}"
    );
}

#[cfg(unix)]
#[test]
fn docker_mount_hint_reports_already_mounted_when_all_paths_are_covered() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let storage_root = temp.path().join("storage");
    let path_dir = temp.path().join("docker-stub-bin");
    fs::create_dir_all(home_dir.join(".claude/skills")).expect("create host claude skills");
    fs::create_dir_all(home_dir.join(".cursor/skills")).expect("create host cursor skills");
    fs::create_dir_all(&path_dir).expect("create docker stub dir");

    let mounts_json = format!(
        r#"[
  {{"Type":"bind","Source":"{}","Destination":"/root/.eden-skills/skills","RW":true}},
  {{"Type":"bind","Source":"{}","Destination":"/root/.claude/skills","RW":true}},
  {{"Type":"bind","Source":"{}","Destination":"/root/.cursor/skills","RW":true}}
]"#,
        storage_root.display(),
        home_dir.join(".claude/skills").display(),
        home_dir.join(".cursor/skills").display()
    );
    write_docker_stub(
        &path_dir.join("docker"),
        temp.path(),
        "my-container",
        "/root",
        &[],
        &mounts_json,
    );
    let config_path = write_mount_hint_config(temp.path(), &storage_root, "my-container");

    let output = eden_command(&home_dir)
        .env("PATH", &path_dir)
        .args(["docker", "mount-hint", "my-container", "--config"])
        .arg(&config_path)
        .output()
        .expect("run docker mount-hint");
    assert_eq!(
        output.status.code(),
        Some(0),
        "docker mount-hint should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("already has all recommended bind mounts"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[cfg(unix)]
#[test]
fn install_with_docker_target_emits_bind_mount_tip_after_docker_cp_fallback() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let path_dir = temp.path().join("docker-stub-bin");
    fs::create_dir_all(&path_dir).expect("create docker stub dir");
    write_docker_stub(
        &path_dir.join("docker"),
        temp.path(),
        "my-container",
        "/root",
        &[".claude"],
        "[]",
    );

    let repo_dir = temp.path().join("docker-tip-repo");
    write_root_skill_repo(&repo_dir, "docker-tip-skill");
    let config_path = temp.path().join("skills.toml");

    let output = eden_command(&home_dir)
        .env("PATH", &path_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./docker-tip-repo",
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

    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains(
            "Tip: add bind mounts for live sync. Run 'eden-skills docker mount-hint my-container'."
        ),
        "combined output={combined}"
    );
}

#[cfg(unix)]
fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

#[cfg(unix)]
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
fn write_mount_hint_config(base: &Path, storage_root: &Path, container_name: &str) -> PathBuf {
    let config_path = base.join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[[skills]]
id = "docker-hint-skill"

[skills.source]
repo = "https://example.com/docker-hint-skill.git"
subpath = "."
ref = "main"

[skills.install]
mode = "copy"

[[skills.targets]]
agent = "claude-code"
environment = "docker:{container_name}"

[[skills.targets]]
agent = "cursor"
environment = "docker:{container_name}"

[skills.verify]
enabled = true
checks = ["path-exists", "content-present"]

[skills.safety]
no_exec_metadata_only = false
"#,
            storage_root = toml_escape_path(storage_root),
            container_name = container_name,
        ),
    )
    .expect("write mount-hint config");
    config_path
}

#[cfg(unix)]
fn write_docker_stub(
    docker_bin: &Path,
    state_root: &Path,
    container_name: &str,
    container_home: &str,
    detected_dirs: &[&str],
    mounts_json: &str,
) {
    let copied_targets = state_root.join("copied-targets.log");
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
        printf '%s' "{container_home}"
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
        container_home = container_home,
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

#[cfg(unix)]
fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}
