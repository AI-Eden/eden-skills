use std::fs;
use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn list_text_prints_inventory() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[storage]
root = "./storage"

[[skills]]
id = "demo"

[skills.source]
repo = "file:///tmp/unused"
subpath = "packages/browser"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "custom"
path = "./targets"
"#,
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["list", "--config"])
        .arg(&config_path)
        .output()
        .expect("run list");

    assert_eq!(
        output.status.code(),
        Some(0),
        "list should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("skill id=demo"), "stdout={stdout}");
    assert!(stdout.contains("mode=symlink"), "stdout={stdout}");
}

#[test]
fn list_json_emits_machine_readable_inventory() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[storage]
root = "./storage"

[[skills]]
id = "demo"

[skills.source]
repo = "file:///tmp/unused"
subpath = "packages/browser"
ref = "main"

[skills.install]
mode = "copy"

[[skills.targets]]
agent = "custom"
path = "./targets"

[skills.verify]
enabled = true
checks = ["path-exists", "content-present"]
"#,
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["list", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run list --json");

    assert_eq!(
        output.status.code(),
        Some(0),
        "list --json should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value =
        serde_json::from_str(&stdout).unwrap_or_else(|err| panic!("invalid json: {err}"));

    assert_eq!(payload["count"].as_u64(), Some(1));
    let skills = payload["skills"].as_array().expect("skills array");
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0]["id"].as_str(), Some("demo"));
    assert_eq!(skills[0]["install"]["mode"].as_str(), Some("copy"));
}
