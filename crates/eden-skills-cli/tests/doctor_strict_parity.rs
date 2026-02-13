mod common;

use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

use common::{as_file_url, init_origin_repo, write_config};

#[test]
fn doctor_text_strict_and_non_strict_emit_equivalent_findings_payload() {
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

    let non_strict = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor non-strict");
    assert_eq!(
        non_strict.status.code(),
        Some(0),
        "doctor non-strict should succeed, stderr={}",
        String::from_utf8_lossy(&non_strict.stderr)
    );

    let strict = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--strict", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor strict");
    assert_eq!(
        strict.status.code(),
        Some(3),
        "doctor strict should exit 3, stderr={}",
        String::from_utf8_lossy(&strict.stderr)
    );

    let non_strict_stdout = String::from_utf8_lossy(&non_strict.stdout).to_string();
    let strict_stdout = String::from_utf8_lossy(&strict.stdout).to_string();
    assert_eq!(
        non_strict_stdout, strict_stdout,
        "strict mode should keep text payload equivalent"
    );
}

#[test]
fn doctor_json_strict_and_non_strict_emit_equivalent_payload_and_required_fields() {
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

    let non_strict = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor --json non-strict");
    assert_eq!(
        non_strict.status.code(),
        Some(0),
        "doctor --json non-strict should succeed, stderr={}",
        String::from_utf8_lossy(&non_strict.stderr)
    );

    let strict = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--strict", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor --json strict");
    assert_eq!(
        strict.status.code(),
        Some(3),
        "doctor --json strict should exit 3, stderr={}",
        String::from_utf8_lossy(&strict.stderr)
    );

    let non_strict_payload: Value =
        serde_json::from_slice(&non_strict.stdout).expect("non-strict stdout should be valid json");
    let strict_payload: Value =
        serde_json::from_slice(&strict.stdout).expect("strict stdout should be valid json");

    assert_eq!(
        non_strict_payload, strict_payload,
        "strict mode should keep json payload equivalent"
    );

    let summary = &non_strict_payload["summary"];
    let total = summary["total"]
        .as_u64()
        .expect("summary.total should be integer");
    let error = summary["error"]
        .as_u64()
        .expect("summary.error should be integer");
    let warning = summary["warning"]
        .as_u64()
        .expect("summary.warning should be integer");
    assert!(total > 0, "expected non-empty findings");
    assert!(error > 0, "expected at least one error finding");
    assert!(warning > 0, "expected at least one warning finding");

    let findings = non_strict_payload["findings"]
        .as_array()
        .expect("findings should be array");
    assert_eq!(
        findings.len() as u64,
        total,
        "summary.total should match findings length"
    );
    for finding in findings {
        assert!(
            finding.get("code").and_then(|v| v.as_str()).is_some(),
            "missing required field `code`: {finding}"
        );
        assert!(
            finding.get("severity").and_then(|v| v.as_str()).is_some(),
            "missing required field `severity`: {finding}"
        );
        assert!(
            finding.get("skill_id").and_then(|v| v.as_str()).is_some(),
            "missing required field `skill_id`: {finding}"
        );
        assert!(
            finding
                .get("target_path")
                .and_then(|v| v.as_str())
                .is_some(),
            "missing required field `target_path`: {finding}"
        );
        assert!(
            finding.get("message").and_then(|v| v.as_str()).is_some(),
            "missing required field `message`: {finding}"
        );
        assert!(
            finding
                .get("remediation")
                .and_then(|v| v.as_str())
                .is_some(),
            "missing required field `remediation`: {finding}"
        );
    }
}
