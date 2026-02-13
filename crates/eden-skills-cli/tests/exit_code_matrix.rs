mod common;

use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

use common::{as_file_url, init_origin_repo, write_config};

#[test]
fn apply_returns_exit_code_1_on_runtime_git_failure() {
    let temp = tempdir().expect("tempdir");

    let missing_repo = temp.path().join("missing-origin-repo");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config(
        temp.path(),
        &as_file_url(&missing_repo),
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["apply", "--config"])
        .arg(&config_path)
        .output()
        .expect("run apply");

    assert_eq!(
        output.status.code(),
        Some(1),
        "expected runtime exit code 1, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn doctor_strict_returns_exit_code_3_on_conflict() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config(
        temp.path(),
        &as_file_url(&origin_repo),
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
    );

    // No apply beforehand: source path under storage root is missing, so doctor sees conflict.
    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--strict", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor strict");

    assert_eq!(
        output.status.code(),
        Some(3),
        "expected strict conflict exit code 3, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
#[test]
fn apply_strict_returns_exit_code_3_on_target_conflict() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config(
        temp.path(),
        &as_file_url(&origin_repo),
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
    );

    let first_apply = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["apply", "--config"])
        .arg(&config_path)
        .output()
        .expect("run first apply");
    assert_eq!(
        first_apply.status.code(),
        Some(0),
        "expected initial apply success, stderr={}",
        String::from_utf8_lossy(&first_apply.stderr)
    );

    let conflicted_target = target_root.join(common::SKILL_ID);
    std::fs::remove_file(&conflicted_target).expect("remove symlink target");
    std::fs::create_dir_all(&conflicted_target).expect("create conflicting directory target");
    std::fs::write(conflicted_target.join("manual.txt"), "manual content").expect("write file");

    let strict_apply = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["apply", "--strict", "--config"])
        .arg(&config_path)
        .output()
        .expect("run strict apply");

    assert_eq!(
        strict_apply.status.code(),
        Some(3),
        "expected strict conflict exit code 3, stderr={}",
        String::from_utf8_lossy(&strict_apply.stderr)
    );
    assert!(
        Path::new(&conflicted_target).exists(),
        "strict apply must not delete unknown conflicting target"
    );
}
