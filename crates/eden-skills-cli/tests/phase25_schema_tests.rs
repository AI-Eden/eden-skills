use std::fs;
use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn plan_with_empty_config_succeeds_and_reports_zero_actions() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[storage]
root = "./storage"
"#,
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["plan", "--config"])
        .arg(&config_path)
        .output()
        .expect("run plan");

    assert_eq!(
        output.status.code(),
        Some(0),
        "plan should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("0 actions"),
        "plan should report zero actions for empty config, stdout={stdout}"
    );
}

#[test]
fn plan_json_with_empty_config_emits_empty_array() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[storage]
root = "./storage"
"#,
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["plan", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run plan --json");

    assert_eq!(
        output.status.code(),
        Some(0),
        "plan --json should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value =
        serde_json::from_str(&stdout).unwrap_or_else(|err| panic!("invalid json: {err}"));
    let items = payload.as_array().expect("plan json array");
    assert!(items.is_empty(), "expected empty plan array, got: {stdout}");
}

#[test]
fn apply_with_empty_config_succeeds_with_zero_summary() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[storage]
root = "./storage"
"#,
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["apply", "--config"])
        .arg(&config_path)
        .output()
        .expect("run apply");

    assert_eq!(
        output.status.code(),
        Some(0),
        "apply should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("apply summary: create=0 update=0 noop=0 conflict=0"),
        "expected zero apply summary for empty config, stdout={stdout}"
    );
}
