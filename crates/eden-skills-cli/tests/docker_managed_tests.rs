#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
use serde_json::Value;
#[cfg(unix)]
use tempfile::tempdir;

#[cfg(unix)]
#[test]
fn tm_p297_037_install_to_docker_target_writes_eden_managed_with_external_source() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let path_dir = temp.path().join("docker-stub-bin");
    let state_dir = temp.path().join("docker-state");
    let host_agent_dir = temp.path().join("host-agent-dir");
    fs::create_dir_all(&path_dir).expect("create path dir");
    fs::create_dir_all(&state_dir).expect("create docker state dir");
    fs::create_dir_all(&host_agent_dir).expect("create host agent dir");

    let mounts_json = format!(
        r#"[
  {{"Type":"bind","Source":"{}","Destination":"/root/.claude/skills","RW":true}}
]"#,
        host_agent_dir.display()
    );
    write_docker_stub(
        &path_dir.join("docker"),
        &state_dir,
        "my-container",
        &[".claude"],
        &mounts_json,
    );

    let repo_dir = temp.path().join("docker-managed-repo");
    write_root_skill_repo(&repo_dir, "docker-managed-skill");
    let config_path = temp.path().join("skills.toml");

    let output = eden_command(&home_dir)
        .env("PATH", &path_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./docker-managed-repo",
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

    let manifest_path = host_agent_dir.join(".eden-managed");
    assert!(
        manifest_path.exists(),
        "docker install should write .eden-managed, stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(&manifest_path).expect("read .eden-managed manifest"),
    )
    .expect("manifest should be valid json");
    assert_eq!(manifest["version"], 1);
    assert_eq!(
        manifest["skills"]["docker-managed-skill"]["source"],
        "external"
    );
}

#[cfg(unix)]
#[test]
fn tm_p297_038_local_install_writes_eden_managed_with_local_source() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(home_dir.join(".claude/skills")).expect("create local claude dir");

    let repo_dir = temp.path().join("local-managed-repo");
    write_root_skill_repo(&repo_dir, "local-managed-skill");
    let config_path = temp.path().join("skills.toml");

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./local-managed-repo", "--config"])
        .arg(&config_path)
        .output()
        .expect("run local install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "local install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest = read_manifest(&home_dir.join(".claude/skills/.eden-managed"));
    assert_eq!(manifest["version"], 1);
    assert_eq!(manifest["skills"]["local-managed-skill"]["source"], "local");
}

#[cfg(unix)]
#[test]
fn tm_p297_039_remove_external_skill_defaults_to_config_only() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let target_root = temp.path().join("agent-skills");
    let storage_root = temp.path().join("storage");
    let config_path =
        write_single_skill_config(temp.path(), &storage_root, &target_root, "guarded-skill");
    let installed_skill = target_root.join("guarded-skill");
    fs::create_dir_all(&installed_skill).expect("create installed skill dir");
    fs::write(installed_skill.join("README.md"), "external version\n")
        .expect("write installed skill file");
    write_manifest(
        &target_root.join(".eden-managed"),
        "guarded-skill",
        "external",
        "host:eden-desktop",
    );

    let output = eden_command(&home_dir)
        .args(["remove", "guarded-skill", "-y", "--config"])
        .arg(&config_path)
        .output()
        .expect("run remove");
    assert_eq!(
        output.status.code(),
        Some(0),
        "remove should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        installed_skill.exists(),
        "config-only removal should keep files"
    );
    assert!(
        read_config_skill_ids(&config_path).is_empty(),
        "skill should be removed from config"
    );
    assert_eq!(
        read_manifest(&target_root.join(".eden-managed"))["skills"]["guarded-skill"]["source"],
        "external"
    );
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("removing from config only"),
        "remove should explain the guard, output={combined}"
    );
}

#[cfg(unix)]
#[test]
fn tm_p297_040_remove_force_external_skill_deletes_files_and_manifest_entry() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let target_root = temp.path().join("agent-skills");
    let storage_root = temp.path().join("storage");
    let config_path =
        write_single_skill_config(temp.path(), &storage_root, &target_root, "forced-skill");
    let installed_skill = target_root.join("forced-skill");
    fs::create_dir_all(&installed_skill).expect("create installed skill dir");
    fs::write(installed_skill.join("README.md"), "external version\n")
        .expect("write installed skill file");
    write_manifest(
        &target_root.join(".eden-managed"),
        "forced-skill",
        "external",
        "host:eden-desktop",
    );

    let output = eden_command(&home_dir)
        .args(["remove", "forced-skill", "-y", "--force", "--config"])
        .arg(&config_path)
        .output()
        .expect("run remove --force");
    assert_eq!(
        output.status.code(),
        Some(0),
        "remove --force should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !installed_skill.exists(),
        "remove --force should delete installed files"
    );
    assert!(
        read_manifest(&target_root.join(".eden-managed"))["skills"]["forced-skill"].is_null(),
        "remove --force should delete manifest entry"
    );
}

#[cfg(unix)]
#[test]
fn tm_p297_041_install_existing_external_skill_warns_and_adopts_without_overwrite() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let target_root = home_dir.join(".claude/skills");
    let installed_skill = target_root.join("install-guard-skill");
    fs::create_dir_all(&installed_skill).expect("create installed skill dir");
    fs::write(installed_skill.join("README.md"), "external version\n")
        .expect("write installed skill file");
    write_manifest(
        &target_root.join(".eden-managed"),
        "install-guard-skill",
        "external",
        "host:eden-desktop",
    );

    let repo_dir = temp.path().join("install-guard-repo");
    write_root_skill_repo(&repo_dir, "install-guard-skill");
    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./install-guard-repo", "--config"])
        .arg(&config_path)
        .output()
        .expect("run guarded install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "guarded install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(installed_skill.join("README.md")).expect("read kept file"),
        "external version\n",
        "guarded install should keep existing files"
    );
    assert_eq!(
        read_manifest(&target_root.join(".eden-managed"))["skills"]["install-guard-skill"]
            ["source"],
        "local"
    );
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("managed by external host"),
        "install should warn before adopt, output={combined}"
    );
}

#[cfg(unix)]
#[test]
fn tm_p297_044_missing_manifest_does_not_block_remove_operation() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let target_root = temp.path().join("agent-skills");
    let storage_root = temp.path().join("storage");
    let config_path = write_single_skill_config(
        temp.path(),
        &storage_root,
        &target_root,
        "missing-manifest-skill",
    );
    let installed_skill = target_root.join("missing-manifest-skill");
    fs::create_dir_all(&installed_skill).expect("create installed skill dir");
    fs::write(installed_skill.join("README.md"), "local version\n")
        .expect("write installed skill file");

    let output = eden_command(&home_dir)
        .args(["remove", "missing-manifest-skill", "-y", "--config"])
        .arg(&config_path)
        .output()
        .expect("run remove without manifest");
    assert_eq!(
        output.status.code(),
        Some(0),
        "remove should succeed without manifest, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !installed_skill.exists(),
        "remove should still delete local files when manifest is missing"
    );
}

#[cfg(unix)]
#[test]
fn tm_p297_045_corrupted_manifest_emits_warning_and_install_proceeds() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let target_root = home_dir.join(".claude/skills");
    fs::create_dir_all(&target_root).expect("create target root");
    fs::write(target_root.join(".eden-managed"), "{not valid json\n").expect("write bad manifest");

    let repo_dir = temp.path().join("corrupted-manifest-repo");
    write_root_skill_repo(&repo_dir, "corrupted-manifest-skill");
    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./corrupted-manifest-repo", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install with corrupted manifest");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed with corrupted manifest, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("Ignoring invalid `.eden-managed` manifest"),
        "install should warn about corrupted manifest, output={combined}"
    );
    assert_eq!(
        read_manifest(&target_root.join(".eden-managed"))["skills"]["corrupted-manifest-skill"]
            ["source"],
        "local"
    );
}

#[cfg(unix)]
#[test]
fn tm_p297_042_doctor_reports_docker_ownership_changed() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let path_dir = temp.path().join("docker-stub-bin");
    let state_dir = temp.path().join("docker-state");
    let host_agent_dir = temp.path().join("host-agent-dir");
    fs::create_dir_all(&path_dir).expect("create path dir");
    fs::create_dir_all(&state_dir).expect("create docker state dir");
    fs::create_dir_all(host_agent_dir.join("doctor-owned-skill"))
        .expect("create managed skill dir");

    let mounts_json = format!(
        r#"[
  {{"Type":"bind","Source":"{}","Destination":"/root/.claude/skills","RW":true}}
]"#,
        host_agent_dir.display()
    );
    write_docker_stub(
        &path_dir.join("docker"),
        &state_dir,
        "my-container",
        &[".claude"],
        &mounts_json,
    );
    write_manifest(
        &host_agent_dir.join(".eden-managed"),
        "doctor-owned-skill",
        "local",
        "container:my-container",
    );
    let config_path = write_docker_target_config(
        temp.path(),
        &temp.path().join("storage"),
        "doctor-owned-skill",
        "my-container",
        "/root/.claude/skills",
    );

    let output = eden_command(&home_dir)
        .env("PATH", &path_dir)
        .args(["doctor", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor --json");
    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid doctor json");
    let findings = payload["findings"].as_array().expect("findings array");
    let finding = findings
        .iter()
        .find(|finding| finding["code"] == "DOCKER_OWNERSHIP_CHANGED")
        .expect("expected DOCKER_OWNERSHIP_CHANGED");
    assert_eq!(finding["severity"], "warning");
    assert_eq!(finding["skill_id"], "doctor-owned-skill");
}

#[cfg(unix)]
#[test]
fn tm_p297_043_doctor_reports_docker_externally_removed() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let path_dir = temp.path().join("docker-stub-bin");
    let state_dir = temp.path().join("docker-state");
    let host_agent_dir = temp.path().join("host-agent-dir");
    fs::create_dir_all(&path_dir).expect("create path dir");
    fs::create_dir_all(&state_dir).expect("create docker state dir");
    fs::create_dir_all(&host_agent_dir).expect("create host agent dir");

    let mounts_json = format!(
        r#"[
  {{"Type":"bind","Source":"{}","Destination":"/root/.claude/skills","RW":true}}
]"#,
        host_agent_dir.display()
    );
    write_docker_stub(
        &path_dir.join("docker"),
        &state_dir,
        "my-container",
        &[".claude"],
        &mounts_json,
    );
    let config_path = write_docker_target_config(
        temp.path(),
        &temp.path().join("storage"),
        "doctor-missing-skill",
        "my-container",
        "/root/.claude/skills",
    );

    let output = eden_command(&home_dir)
        .env("PATH", &path_dir)
        .args(["doctor", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor --json");
    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid doctor json");
    let findings = payload["findings"].as_array().expect("findings array");
    let finding = findings
        .iter()
        .find(|finding| finding["code"] == "DOCKER_EXTERNALLY_REMOVED")
        .expect("expected DOCKER_EXTERNALLY_REMOVED");
    assert_eq!(finding["severity"], "warning");
    assert_eq!(finding["skill_id"], "doctor-missing-skill");
}

#[cfg(unix)]
#[test]
fn tm_p297_046_apply_force_reclaims_ownership_from_local_back_to_external() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let path_dir = temp.path().join("docker-stub-bin");
    let state_dir = temp.path().join("docker-state");
    let host_agent_dir = temp.path().join("host-agent-dir");
    fs::create_dir_all(&path_dir).expect("create path dir");
    fs::create_dir_all(&state_dir).expect("create docker state dir");
    fs::create_dir_all(host_agent_dir.join("reclaim-skill")).expect("create managed skill dir");

    let mounts_json = format!(
        r#"[
  {{"Type":"bind","Source":"{}","Destination":"{}","RW":true}}
]"#,
        host_agent_dir.display(),
        host_agent_dir.display()
    );
    write_docker_stub(
        &path_dir.join("docker"),
        &state_dir,
        "my-container",
        &[".claude"],
        &mounts_json,
    );
    write_manifest(
        &host_agent_dir.join(".eden-managed"),
        "reclaim-skill",
        "local",
        "container:my-container",
    );

    let repo_dir = temp.path().join("reclaim-repo");
    init_apply_repo(&repo_dir, "reclaim-skill");
    let config_path = write_docker_apply_config(
        temp.path(),
        &temp.path().join("storage"),
        &host_agent_dir,
        &as_file_url(&repo_dir),
        "reclaim-skill",
        "my-container",
    );

    let output = eden_command(&home_dir)
        .env("PATH", prepend_path(&path_dir))
        .args(["apply", "--force", "--config"])
        .arg(&config_path)
        .output()
        .expect("run apply --force");
    assert_eq!(
        output.status.code(),
        Some(0),
        "apply --force should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        read_manifest(&host_agent_dir.join(".eden-managed"))["skills"]["reclaim-skill"]["source"],
        "external"
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

#[cfg(unix)]
fn write_single_skill_config(
    base: &Path,
    storage_root: &Path,
    target_root: &Path,
    skill_id: &str,
) -> PathBuf {
    fs::create_dir_all(storage_root).expect("create storage root");
    fs::create_dir_all(target_root).expect("create target root");
    let config_path = base.join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[[skills]]
id = "{skill_id}"

[skills.source]
repo = "file:///tmp/example"
subpath = "."
ref = "main"

[skills.install]
mode = "copy"

[[skills.targets]]
agent = "custom"
path = "{target_root}"

[skills.verify]
enabled = true
checks = ["path-exists", "content-present"]

[skills.safety]
no_exec_metadata_only = false
"#,
            storage_root = toml_escape_path(storage_root),
            skill_id = skill_id,
            target_root = toml_escape_path(target_root),
        ),
    )
    .expect("write config");
    config_path
}

#[cfg(unix)]
fn write_docker_target_config(
    base: &Path,
    storage_root: &Path,
    skill_id: &str,
    container_name: &str,
    target_root: &str,
) -> PathBuf {
    fs::create_dir_all(storage_root).expect("create storage root");
    let config_path = base.join(format!("{skill_id}.toml"));
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[[skills]]
id = "{skill_id}"

[skills.source]
repo = "file:///tmp/example"
subpath = "."
ref = "main"

[skills.install]
mode = "copy"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
environment = "docker:{container_name}"

[skills.verify]
enabled = true
checks = ["path-exists", "content-present"]

[skills.safety]
no_exec_metadata_only = false
"#,
            storage_root = toml_escape_path(storage_root),
            skill_id = skill_id,
            target_root = target_root,
            container_name = container_name,
        ),
    )
    .expect("write docker config");
    config_path
}

#[cfg(unix)]
fn write_docker_apply_config(
    base: &Path,
    storage_root: &Path,
    target_root: &Path,
    repo_url: &str,
    skill_id: &str,
    container_name: &str,
) -> PathBuf {
    fs::create_dir_all(storage_root).expect("create storage root");
    let config_path = base.join(format!("{skill_id}-apply.toml"));
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[[skills]]
id = "{skill_id}"

[skills.source]
repo = "{repo_url}"
subpath = "."
ref = "main"

[skills.install]
mode = "copy"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
environment = "docker:{container_name}"

[skills.verify]
enabled = true
checks = ["path-exists", "content-present"]

[skills.safety]
no_exec_metadata_only = false
"#,
            storage_root = toml_escape_path(storage_root),
            skill_id = skill_id,
            repo_url = repo_url,
            target_root = toml_escape_path(target_root),
            container_name = container_name,
        ),
    )
    .expect("write apply config");
    config_path
}

#[cfg(unix)]
fn write_manifest(manifest_path: &Path, skill_id: &str, source: &str, origin: &str) {
    let contents = format!(
        r#"{{
  "version": 1,
  "skills": {{
    "{skill_id}": {{
      "source": "{source}",
      "origin": "{origin}",
      "installed_at": "2026-03-07T10:30:00Z"
    }}
  }}
}}
"#
    );
    fs::write(manifest_path, contents).expect("write manifest");
}

#[cfg(unix)]
fn init_apply_repo(repo_dir: &Path, name: &str) {
    fs::create_dir_all(repo_dir).expect("create repo dir");
    fs::write(
        repo_dir.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: test\n---\n"),
    )
    .expect("write SKILL.md");
    fs::write(repo_dir.join("README.md"), "repo version\n").expect("write README");
    run_git(repo_dir, &["init"]);
    run_git(repo_dir, &["config", "user.email", "test@example.com"]);
    run_git(repo_dir, &["config", "user.name", "eden-skills-test"]);
    run_git(repo_dir, &["add", "."]);
    run_git(repo_dir, &["commit", "-m", "init"]);
    run_git(repo_dir, &["branch", "-M", "main"]);
}

#[cfg(unix)]
fn read_manifest(manifest_path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(manifest_path).expect("read manifest"))
        .expect("manifest should be valid json")
}

#[cfg(unix)]
fn read_config_skill_ids(config_path: &Path) -> Vec<String> {
    let config_text = fs::read_to_string(config_path).expect("read config");
    let value: toml::Value = toml::from_str(&config_text).expect("valid config toml");
    value
        .get("skills")
        .and_then(|skills| skills.as_array())
        .map(|skills| {
            skills
                .iter()
                .filter_map(|skill| skill.get("id").and_then(|id| id.as_str()))
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[cfg(unix)]
fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

#[cfg(unix)]
fn as_file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

#[cfg(unix)]
fn run_git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn git");
    assert!(
        output.status.success(),
        "git {:?} failed in {}: status={} stderr=`{}` stdout=`{}`",
        args,
        cwd.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr).trim(),
        String::from_utf8_lossy(&output.stdout).trim()
    );
}

#[cfg(unix)]
fn prepend_path(path_dir: &Path) -> String {
    match std::env::var("PATH") {
        Ok(existing) if !existing.is_empty() => format!("{}:{existing}", path_dir.display()),
        _ => path_dir.display().to_string(),
    }
}
