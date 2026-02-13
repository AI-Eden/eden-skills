use std::fs;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn config_export_emits_valid_toml_and_is_deterministic() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[[skills]]
id = "x"

[skills.source]
repo = "https://github.com/vercel-labs/skills.git"

[[skills.targets]]
agent = "claude-code"
"#,
    )
    .expect("write config");

    let out1 = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["config", "export", "--config"])
        .arg(&config_path)
        .output()
        .expect("run config export");
    assert_eq!(
        out1.status.code(),
        Some(0),
        "config export should succeed, stderr={}",
        String::from_utf8_lossy(&out1.stderr)
    );

    let stdout1 = String::from_utf8_lossy(&out1.stdout).to_string();
    let value1: toml::Value =
        toml::from_str(&stdout1).expect("config export should emit valid toml");
    assert_eq!(value1["version"].as_integer(), Some(1));
    assert!(value1.get("storage").is_some(), "expected storage section");
    assert!(value1.get("skills").is_some(), "expected skills array");

    let out2 = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["config", "export", "--config"])
        .arg(&config_path)
        .output()
        .expect("run config export (2)");
    assert_eq!(out2.status.code(), Some(0));
    let stdout2 = String::from_utf8_lossy(&out2.stdout).to_string();
    assert_eq!(stdout1, stdout2, "config export must be deterministic");
}

#[test]
fn config_export_json_wraps_toml_string() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[[skills]]
id = "x"

[skills.source]
repo = "https://github.com/vercel-labs/skills.git"

[[skills.targets]]
agent = "claude-code"
"#,
    )
    .expect("write config");

    let out = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["config", "export", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run config export --json");
    assert_eq!(out.status.code(), Some(0));

    let payload: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("config export --json should emit valid json");
    assert_eq!(payload["format"].as_str(), Some("toml"));
    assert!(payload["toml"]
        .as_str()
        .unwrap_or("")
        .contains("version = 1"));
}
