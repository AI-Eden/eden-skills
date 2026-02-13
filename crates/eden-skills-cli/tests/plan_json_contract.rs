mod common;

use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

use common::{as_file_url, init_origin_repo, write_config};

#[test]
fn plan_json_has_required_schema() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());

    // Intentionally do not run `apply` before `plan`.
    // This produces `action=conflict` due to missing source path, but schema must still hold.
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
        .args(["plan", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run plan --json");
    assert_eq!(
        output.status.code(),
        Some(0),
        "plan --json should exit 0, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value =
        serde_json::from_str(&stdout).unwrap_or_else(|err| panic!("invalid json: {err}"));

    let entries = payload.as_array().expect("plan json must be an array");
    assert!(!entries.is_empty(), "expected at least one plan entry");

    for entry in entries {
        let obj = entry.as_object().expect("plan entry must be an object");
        for key in [
            "skill_id",
            "source_path",
            "target_path",
            "install_mode",
            "action",
        ] {
            assert!(
                obj.get(key).and_then(|v| v.as_str()).is_some(),
                "entry.{key} must be a string"
            );
        }

        let install_mode = obj
            .get("install_mode")
            .and_then(|v| v.as_str())
            .expect("install_mode str");
        assert!(
            matches!(install_mode, "symlink" | "copy"),
            "install_mode must be symlink|copy"
        );

        let action = obj
            .get("action")
            .and_then(|v| v.as_str())
            .expect("action str");
        assert!(
            matches!(action, "create" | "update" | "noop" | "conflict"),
            "action must be create|update|noop|conflict"
        );

        let reasons = obj
            .get("reasons")
            .and_then(|v| v.as_array())
            .expect("reasons must be an array");
        for reason in reasons {
            assert!(reason.as_str().is_some(), "reasons entries must be strings");
        }
    }
}
