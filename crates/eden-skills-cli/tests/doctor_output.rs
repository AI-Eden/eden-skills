mod common;

use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

use common::{as_file_url, init_origin_repo, write_config};

#[test]
fn doctor_text_output_includes_code_severity_and_remediation() {
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

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor");

    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor should not fail without --strict, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("code=SOURCE_MISSING"),
        "stdout should contain issue code, stdout={stdout}"
    );
    assert!(
        stdout.contains("severity=error"),
        "stdout should contain severity, stdout={stdout}"
    );
    assert!(
        stdout.contains("remediation="),
        "stdout should contain remediation hint, stdout={stdout}"
    );
}

#[test]
fn doctor_json_output_includes_code_severity_and_remediation() {
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

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor json");

    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor --json should not fail without --strict, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(&stdout).unwrap_or_else(|err| {
        panic!("doctor --json should emit valid JSON ({err}), stdout={stdout}")
    });

    let total = payload["summary"]["total"]
        .as_u64()
        .expect("summary.total should be u64");
    assert!(total > 0, "expected at least one finding");

    let findings = payload["findings"]
        .as_array()
        .expect("findings should be an array");
    assert!(
        !findings.is_empty(),
        "expected findings array to be non-empty"
    );

    let has_source_missing = findings.iter().any(|finding| {
        finding["code"] == "SOURCE_MISSING"
            && finding.get("severity").is_some()
            && finding.get("remediation").is_some()
    });
    assert!(
        has_source_missing,
        "expected SOURCE_MISSING finding with severity/remediation, payload={payload}"
    );
}
