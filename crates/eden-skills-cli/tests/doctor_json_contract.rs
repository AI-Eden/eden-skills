mod common;

use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

use common::{as_file_url, init_origin_repo, write_config};

#[test]
fn doctor_json_has_required_schema() {
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
        .expect("run doctor --json");
    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor --json should exit 0 without --strict, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value =
        serde_json::from_str(&stdout).unwrap_or_else(|err| panic!("invalid json: {err}"));

    let summary = payload
        .get("summary")
        .and_then(|v| v.as_object())
        .expect("summary must be an object");
    assert!(
        summary.get("total").and_then(|v| v.as_u64()).is_some(),
        "summary.total must be an integer"
    );
    assert!(
        summary.get("error").and_then(|v| v.as_u64()).is_some(),
        "summary.error must be an integer"
    );
    assert!(
        summary.get("warning").and_then(|v| v.as_u64()).is_some(),
        "summary.warning must be an integer"
    );

    let findings = payload
        .get("findings")
        .and_then(|v| v.as_array())
        .expect("findings must be an array");
    assert!(
        !findings.is_empty(),
        "findings must be non-empty for this setup"
    );

    for finding in findings {
        let obj = finding.as_object().expect("finding must be an object");
        for key in [
            "code",
            "severity",
            "skill_id",
            "target_path",
            "message",
            "remediation",
        ] {
            assert!(
                obj.get(key).and_then(|v| v.as_str()).is_some(),
                "finding.{key} must be a string"
            );
        }

        let severity = obj
            .get("severity")
            .and_then(|v| v.as_str())
            .expect("severity str");
        assert!(
            matches!(severity, "error" | "warning"),
            "severity must be error|warning"
        );
    }
}
