mod common;

use std::fs;
use std::path::{Path, PathBuf};
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("source sync failed for 1 skill(s):"),
        "stderr should include source sync failure summary, got: {stderr}"
    );
    assert!(
        stderr.contains("skill=demo-skill"),
        "stderr should include skill identifier, got: {stderr}"
    );
    assert!(
        stderr.contains("stage=clone"),
        "stderr should include clone failure stage, got: {stderr}"
    );
}

#[test]
fn apply_reports_skipped_source_sync_on_repeated_run() {
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
        "expected first apply success, stderr={}",
        String::from_utf8_lossy(&first_apply.stderr)
    );

    let second_apply = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["apply", "--config"])
        .arg(&config_path)
        .output()
        .expect("run second apply");
    assert_eq!(
        second_apply.status.code(),
        Some(0),
        "expected second apply success, stderr={}",
        String::from_utf8_lossy(&second_apply.stderr)
    );
    let stdout = String::from_utf8_lossy(&second_apply.stdout);
    assert!(
        stdout.contains("source sync: cloned=0 updated=0 skipped=1 failed=0"),
        "expected skipped source sync summary, got: {stdout}"
    );
}

#[test]
fn apply_returns_exit_code_1_with_fetch_failure_diagnostics() {
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

    let fake_repo_dir = storage_root.join(common::SKILL_ID);
    fs::create_dir_all(&fake_repo_dir).expect("create fake repo dir");
    fs::write(fake_repo_dir.join(".git"), "not-a-repo").expect("write fake .git marker");

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

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("skill=demo-skill"),
        "stderr should include skill identifier, got: {stderr}"
    );
    assert!(
        stderr.contains("stage=fetch"),
        "stderr should include fetch failure stage, got: {stderr}"
    );
    assert!(
        stderr.contains(&format!("repo_dir={}", fake_repo_dir.display())),
        "stderr should include repo directory, got: {stderr}"
    );
}

#[test]
fn apply_returns_exit_code_1_with_checkout_failure_diagnostics() {
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

    let config_text = fs::read_to_string(&config_path).expect("read config");
    let invalid_ref_config = config_text.replace("ref = \"main\"", "ref = \"missing-ref\"");
    fs::write(&config_path, invalid_ref_config).expect("rewrite config with invalid ref");

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

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("skill=demo-skill"),
        "stderr should include skill identifier, got: {stderr}"
    );
    assert!(
        stderr.contains("stage=checkout"),
        "stderr should include checkout failure stage, got: {stderr}"
    );
}

#[test]
fn apply_strict_conflict_takes_precedence_over_verify_failure() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config(
        temp.path(),
        &as_file_url(&origin_repo),
        "symlink",
        &["content-present"],
        &storage_root,
        &target_root,
    );

    let conflicted_target = target_root.join(common::SKILL_ID);
    fs::create_dir_all(&conflicted_target).expect("create conflicting directory target");
    fs::write(conflicted_target.join("manual.txt"), "manual content").expect("write file");

    let strict_apply = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["apply", "--strict", "--config"])
        .arg(&config_path)
        .output()
        .expect("run strict apply");

    assert_eq!(
        strict_apply.status.code(),
        Some(3),
        "strict conflict should take precedence over verify failure, stderr={}",
        String::from_utf8_lossy(&strict_apply.stderr)
    );
    assert!(
        String::from_utf8_lossy(&strict_apply.stderr).contains("strict mode blocked apply"),
        "stderr should include strict conflict message"
    );
}

#[test]
fn repair_strict_conflict_takes_precedence_over_verify_failure() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config(
        temp.path(),
        &as_file_url(&origin_repo),
        "symlink",
        &["content-present"],
        &storage_root,
        &target_root,
    );

    let conflicted_target = target_root.join(common::SKILL_ID);
    fs::create_dir_all(&conflicted_target).expect("create conflicting directory target");
    fs::write(conflicted_target.join("manual.txt"), "manual content").expect("write file");

    let strict_repair = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["repair", "--strict", "--config"])
        .arg(&config_path)
        .output()
        .expect("run strict repair");

    assert_eq!(
        strict_repair.status.code(),
        Some(3),
        "strict conflict should take precedence over verify failure, stderr={}",
        String::from_utf8_lossy(&strict_repair.stderr)
    );
    assert!(
        String::from_utf8_lossy(&strict_repair.stderr)
            .contains("repair skipped 1 conflict entries in strict mode"),
        "stderr should include strict conflict message"
    );
}

#[test]
fn apply_aggregates_multiskill_source_failures_in_config_order() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");

    let missing_a = temp.path().join("missing-a");
    let missing_b = temp.path().join("missing-b");
    let missing_a_url = as_file_url(&missing_a);
    let missing_b_url = as_file_url(&missing_b);
    let config_path = write_multiskill_config(
        temp.path(),
        &storage_root,
        &target_root,
        &[
            ("alpha-skill", &missing_a_url),
            ("beta-skill", &missing_b_url),
        ],
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("source sync: cloned=0 updated=0 skipped=0 failed=2"),
        "expected deterministic mixed summary, got: {stdout}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let alpha_pos = stderr
        .find("skill=alpha-skill")
        .expect("alpha-skill diagnostic");
    let beta_pos = stderr
        .find("skill=beta-skill")
        .expect("beta-skill diagnostic");
    assert!(
        alpha_pos < beta_pos,
        "failure diagnostics should preserve config order, stderr={stderr}"
    );
}

#[test]
fn apply_strict_source_sync_failure_takes_precedence_over_conflict_exit_code() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let missing_repo = temp.path().join("missing-origin");
    let origin_repo_url = as_file_url(&origin_repo);
    let missing_repo_url = as_file_url(&missing_repo);

    let config_path = write_multiskill_config(
        temp.path(),
        &storage_root,
        &target_root,
        &[
            ("good-skill", &origin_repo_url),
            ("bad-skill", &missing_repo_url),
        ],
    );

    let conflicted_target = target_root.join("good-skill");
    fs::create_dir_all(&conflicted_target).expect("create conflicting target");
    fs::write(conflicted_target.join("manual.txt"), "manual content").expect("write manual file");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["apply", "--strict", "--config"])
        .arg(&config_path)
        .output()
        .expect("run strict apply");

    assert_eq!(
        output.status.code(),
        Some(1),
        "source sync runtime failure should take precedence over strict conflict exit code, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("source sync failed for 1 skill(s):"),
        "expected source sync failure diagnostics in stderr"
    );
    assert!(
        conflicted_target.join("manual.txt").exists(),
        "strict apply should not mutate conflicting target when source sync fails first"
    );
}

#[test]
fn repair_strict_source_sync_failure_takes_precedence_over_conflict_exit_code() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let missing_repo = temp.path().join("missing-origin");
    let origin_repo_url = as_file_url(&origin_repo);
    let missing_repo_url = as_file_url(&missing_repo);

    let config_path = write_multiskill_config(
        temp.path(),
        &storage_root,
        &target_root,
        &[
            ("good-skill", &origin_repo_url),
            ("bad-skill", &missing_repo_url),
        ],
    );

    let conflicted_target = target_root.join("good-skill");
    fs::create_dir_all(&conflicted_target).expect("create conflicting target");
    fs::write(conflicted_target.join("manual.txt"), "manual content").expect("write manual file");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["repair", "--strict", "--config"])
        .arg(&config_path)
        .output()
        .expect("run strict repair");

    assert_eq!(
        output.status.code(),
        Some(1),
        "source sync runtime failure should take precedence over strict conflict exit code, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("source sync failed for 1 skill(s):"),
        "expected source sync failure diagnostics in stderr"
    );
    assert!(
        conflicted_target.join("manual.txt").exists(),
        "strict repair should not mutate conflicting target when source sync fails first"
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

#[cfg(windows)]
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

fn write_multiskill_config(
    base: &Path,
    storage_root: &Path,
    target_root: &Path,
    skills: &[(&str, &str)],
) -> PathBuf {
    let mut content = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n\n",
        toml_escape_path(storage_root)
    );

    for (id, repo_url) in skills {
        content.push_str("[[skills]]\n");
        content.push_str(&format!("id = \"{}\"\n\n", toml_escape_str(id)));
        content.push_str("[skills.source]\n");
        content.push_str(&format!("repo = \"{}\"\n", toml_escape_str(repo_url)));
        content.push_str("subpath = \"packages/browser\"\n");
        content.push_str("ref = \"main\"\n\n");
        content.push_str("[skills.install]\n");
        content.push_str("mode = \"symlink\"\n\n");
        content.push_str("[[skills.targets]]\n");
        content.push_str("agent = \"custom\"\n");
        content.push_str(&format!("path = \"{}\"\n\n", toml_escape_path(target_root)));
        content.push_str("[skills.verify]\n");
        content.push_str("enabled = true\n");
        content.push_str("checks = [\"path-exists\", \"target-resolves\", \"is-symlink\"]\n\n");
        content.push_str("[skills.safety]\n");
        content.push_str("no_exec_metadata_only = false\n\n");
    }

    let config_path = base.join("skills.toml");
    fs::write(&config_path, content).expect("write config");
    config_path
}

fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

fn toml_escape_str(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
