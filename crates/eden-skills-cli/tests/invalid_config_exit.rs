use std::fs;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn invalid_config_returns_exit_code_2_and_field_path() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("invalid-skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[[skills]]
id = "broken-skill"

[skills.source]
repo = "https://github.com/vercel-labs/skills.git"

[[skills.targets]]
agent = "custom"
"#,
    )
    .expect("write invalid config");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["plan", "--config"])
        .arg(&config_path)
        .output()
        .expect("run eden-skills");

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit code 2, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("skills[0].targets[0].path"),
        "expected field path in stderr, stderr={stderr}"
    );
}

#[test]
fn strict_mode_unknown_top_level_key_returns_exit_code_2() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("strict-invalid-skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1
unexpected_key = true

[[skills]]
id = "ok"

[skills.source]
repo = "https://github.com/vercel-labs/skills.git"

[[skills.targets]]
agent = "claude-code"
"#,
    )
    .expect("write strict-invalid config");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["plan", "--strict", "--config"])
        .arg(&config_path)
        .output()
        .expect("run eden-skills");

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit code 2, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown top-level keys"),
        "expected strict unknown-key error, stderr={stderr}"
    );
}
