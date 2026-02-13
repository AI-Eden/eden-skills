mod common;

use std::fs;
use std::process::Command;

use eden_skills_cli::commands::{apply, CommandOptions};
use eden_skills_core::config::InstallMode;
use serde_json::Value;
use tempfile::tempdir;

use common::{
    as_file_url, default_options, expected_source_path, expected_target_path, init_origin_repo,
    run_git_cmd, write_config_with_safety, SKILL_ID,
};

#[test]
fn apply_no_exec_metadata_only_skips_target_mutation_and_writes_metadata() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let script_path = origin_repo.join("packages").join("browser").join("run.sh");
    fs::write(&script_path, "#!/bin/sh\necho hi\n").expect("write script");
    run_git_cmd(&origin_repo, &["add", "."]);
    run_git_cmd(&origin_repo, &["commit", "-m", "add script"]);

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config_with_safety(
        temp.path(),
        &as_file_url(&origin_repo),
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
        true,
    );

    apply(
        config_path.to_str().expect("config path"),
        default_options(),
    )
    .expect("apply with no_exec_metadata_only");

    let target_path = expected_target_path(&target_root);
    assert!(
        !target_path.exists(),
        "target should not be created when no_exec_metadata_only=true"
    );

    let source_path = expected_source_path(&storage_root);
    assert!(source_path.exists(), "source should still be synchronized");

    let metadata_path = storage_root.join(SKILL_ID).join(".eden-safety.toml");
    let metadata = fs::read_to_string(&metadata_path).expect("read safety metadata");
    assert!(metadata.contains("version = 1"));
    assert!(metadata.contains("no_exec_metadata_only = true"));
    assert!(metadata.contains("license_status = \"unknown\""));
    assert!(metadata.contains("contains-shell-script"));
}

#[test]
fn doctor_reports_safety_findings() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let script_path = origin_repo.join("packages").join("browser").join("run.sh");
    fs::write(&script_path, "#!/bin/sh\necho hi\n").expect("write script");
    run_git_cmd(&origin_repo, &["add", "."]);
    run_git_cmd(&origin_repo, &["commit", "-m", "add script"]);

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config_with_safety(
        temp.path(),
        &as_file_url(&origin_repo),
        InstallMode::Symlink.as_str(),
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
        true,
    );

    apply(
        config_path.to_str().expect("config path"),
        CommandOptions {
            strict: false,
            json: false,
        },
    )
    .expect("apply before doctor");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor --json");

    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor should succeed without strict mode, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("doctor should output json");
    let findings = payload["findings"]
        .as_array()
        .expect("findings should be an array");

    let has_license_unknown = findings
        .iter()
        .any(|f| f["code"] == "LICENSE_UNKNOWN" && f["severity"] == "warning");
    let has_risk_review = findings
        .iter()
        .any(|f| f["code"] == "RISK_REVIEW_REQUIRED" && f["severity"] == "warning");
    let has_no_exec = findings
        .iter()
        .any(|f| f["code"] == "NO_EXEC_METADATA_ONLY" && f["severity"] == "warning");

    assert!(has_license_unknown, "expected LICENSE_UNKNOWN in findings");
    assert!(has_risk_review, "expected RISK_REVIEW_REQUIRED in findings");
    assert!(has_no_exec, "expected NO_EXEC_METADATA_ONLY in findings");
}
