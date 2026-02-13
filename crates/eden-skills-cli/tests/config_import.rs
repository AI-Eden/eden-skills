use std::fs;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn config_import_dry_run_emits_toml_and_does_not_create_dest() {
    let temp = tempdir().expect("tempdir");
    let from_path = temp.path().join("from.toml");
    let dest_path = temp.path().join("dest.toml");

    fs::write(
        &from_path,
        r#"
version = 1

[[skills]]
id = "x"

[skills.source]
repo = "https://example.com/repo.git"

[[skills.targets]]
agent = "claude-code"
"#,
    )
    .expect("write from config");

    let out = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["config", "import", "--from"])
        .arg(&from_path)
        .args(["--config"])
        .arg(&dest_path)
        .args(["--dry-run"])
        .output()
        .expect("run config import --dry-run");

    assert_eq!(
        out.status.code(),
        Some(0),
        "config import --dry-run should succeed, stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let value: toml::Value =
        toml::from_str(&stdout).expect("config import --dry-run should emit valid toml");
    assert_eq!(value["version"].as_integer(), Some(1));
    assert!(
        value.get("storage").is_some(),
        "normalized output should include storage section"
    );
    assert!(
        !dest_path.exists(),
        "config import --dry-run must not create destination file"
    );
}

#[test]
fn config_import_writes_normalized_toml_to_destination() {
    let temp = tempdir().expect("tempdir");
    let from_path = temp.path().join("from.toml");
    let dest_path = temp.path().join("dest.toml");

    fs::write(
        &from_path,
        r#"
version = 1

[[skills]]
id = "x"

[skills.source]
repo = "https://example.com/repo.git"

[[skills.targets]]
agent = "claude-code"
"#,
    )
    .expect("write from config");

    let out = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["config", "import", "--from"])
        .arg(&from_path)
        .args(["--config"])
        .arg(&dest_path)
        .output()
        .expect("run config import");

    assert_eq!(
        out.status.code(),
        Some(0),
        "config import should succeed, stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let written = fs::read_to_string(&dest_path).expect("read dest");
    let value: toml::Value = toml::from_str(&written).expect("dest should contain valid toml");
    assert_eq!(value["version"].as_integer(), Some(1));
    assert!(
        value.get("storage").is_some(),
        "normalized output should include storage section"
    );
    assert!(
        value.get("skills").is_some(),
        "normalized output should include skills array"
    );
}

#[test]
fn config_import_strict_rejects_unknown_top_level_keys() {
    let temp = tempdir().expect("tempdir");
    let from_path = temp.path().join("from.toml");
    let dest_path = temp.path().join("dest.toml");

    fs::write(
        &from_path,
        r#"
version = 1
extra = true

[[skills]]
id = "x"

[skills.source]
repo = "https://example.com/repo.git"

[[skills.targets]]
agent = "claude-code"
"#,
    )
    .expect("write from config");

    let out = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["config", "import", "--strict", "--from"])
        .arg(&from_path)
        .args(["--config"])
        .arg(&dest_path)
        .output()
        .expect("run config import --strict");

    assert_eq!(
        out.status.code(),
        Some(2),
        "config import --strict with unknown keys should exit 2, stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !dest_path.exists(),
        "config import --strict should not write destination on validation error"
    );
}
