use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tempfile::tempdir;

#[test]
fn doctor_emits_registry_stale_when_last_sync_is_older_than_7_days() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let config_path = temp.path().join("skills.toml");

    fs::create_dir_all(storage_root.join("registries").join("official"))
        .expect("create registry cache dir");
    let stale_timestamp = SystemTime::now()
        .checked_sub(Duration::from_secs(8 * 24 * 60 * 60))
        .expect("stale timestamp")
        .duration_since(UNIX_EPOCH)
        .expect("duration since epoch")
        .as_secs();
    fs::write(
        storage_root
            .join("registries")
            .join("official")
            .join(".eden-last-sync"),
        stale_timestamp.to_string(),
    )
    .expect("write stale marker");

    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[registries]
official = {{ url = "https://example.com/official.git", priority = 100 }}

[[skills]]
id = "demo"

[skills.source]
repo = "https://example.com/demo.git"
subpath = "."
ref = "main"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor");

    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor should succeed without --strict, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("REGISTRY_STALE"),
        "expected REGISTRY_STALE in doctor output, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn doctor_emits_docker_not_found_for_docker_targets() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let config_path = write_docker_target_config(temp.path(), &storage_root, &target_root);
    let docker_stub_dir = temp.path().join("docker-unavailable-bin");
    fs::create_dir_all(&docker_stub_dir).expect("create docker stub dir");
    write_unavailable_docker_stub(&docker_stub_dir);

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--config"])
        .arg(&config_path)
        .env("PATH", &docker_stub_dir)
        // Keep resolution deterministic even when runner images preinstall docker.
        .current_dir(&docker_stub_dir)
        .output()
        .expect("run doctor");

    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor should succeed without --strict, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("DOCKER_NOT_FOUND"),
        "expected DOCKER_NOT_FOUND in doctor output, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[cfg(unix)]
#[test]
fn doctor_emits_adapter_health_fail_when_container_is_not_running() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let config_path = write_docker_target_config(temp.path(), &storage_root, &target_root);
    let path_dir = temp.path().join("docker-stub-bin");
    fs::create_dir_all(&path_dir).expect("create path dir");

    let docker_bin = path_dir.join("docker");
    let script = r#"#!/bin/sh
set -eu
cmd="$1"
shift
if [ "$cmd" = "--version" ]; then
  echo "Docker version 27.0.0"
  exit 0
fi
if [ "$cmd" = "inspect" ]; then
  echo "false"
  exit 0
fi
echo "unsupported docker call" >&2
exit 1
"#;
    fs::write(&docker_bin, script).expect("write docker stub");
    let mut perms = fs::metadata(&docker_bin)
        .expect("docker stub metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&docker_bin, perms).expect("set docker stub executable");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--config"])
        .arg(&config_path)
        .env("PATH", &path_dir)
        .output()
        .expect("run doctor");

    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor should succeed without --strict, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("ADAPTER_HEALTH_FAIL"),
        "expected ADAPTER_HEALTH_FAIL in doctor output, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

fn write_docker_target_config(base: &Path, storage_root: &Path, target_root: &Path) -> PathBuf {
    let config_path = base.join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[[skills]]
id = "docker-skill"

[skills.source]
repo = "https://example.com/docker-skill.git"
subpath = "."
ref = "main"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
environment = "docker:test-container"
"#,
            storage_root = toml_escape_path(storage_root),
            target_root = toml_escape_path(target_root),
        ),
    )
    .expect("write config");
    config_path
}

#[cfg(unix)]
fn write_unavailable_docker_stub(path_dir: &Path) {
    let docker_bin = path_dir.join("docker");
    let script = r#"#!/bin/sh
exit 1
"#;
    fs::write(&docker_bin, script).expect("write docker unavailable stub");
    let mut perms = fs::metadata(&docker_bin)
        .expect("docker unavailable stub metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&docker_bin, perms).expect("set docker unavailable stub executable");
}

#[cfg(windows)]
fn write_unavailable_docker_stub(path_dir: &Path) {
    // std::process::Command resolves `docker` to `docker.exe` on Windows.
    // Write a deliberately invalid PE file so spawning it fails deterministically.
    let docker_bin = path_dir.join("docker.exe");
    fs::write(&docker_bin, b"not-a-valid-exe").expect("write docker unavailable stub");
}

fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}
