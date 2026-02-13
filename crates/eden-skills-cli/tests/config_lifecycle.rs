use std::fs;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn add_appends_skill_and_writes_valid_toml() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");

    let init = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    assert_eq!(
        init.status.code(),
        Some(0),
        "init should succeed, stderr={}",
        String::from_utf8_lossy(&init.stderr)
    );

    let add = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args([
            "add",
            "--config",
            config_path.to_str().expect("utf8 path"),
            "--id",
            "new-skill",
            "--repo",
            "https://example.com/repo.git",
            "--target",
            "claude-code",
        ])
        .output()
        .expect("run add");
    assert_eq!(
        add.status.code(),
        Some(0),
        "add should succeed, stderr={}",
        String::from_utf8_lossy(&add.stderr)
    );

    let written = fs::read_to_string(&config_path).expect("read config");
    let value: toml::Value = toml::from_str(&written).expect("config should be valid toml");

    let skills = value
        .get("skills")
        .and_then(|v| v.as_array())
        .expect("skills should be an array");
    assert_eq!(skills.len(), 2);
    assert_eq!(skills[0]["id"].as_str(), Some("browser-tool"));
    assert_eq!(skills[1]["id"].as_str(), Some("new-skill"));
    assert_eq!(
        skills[1]["source"]["repo"].as_str(),
        Some("https://example.com/repo.git")
    );
    assert_eq!(skills[1]["source"]["ref"].as_str(), Some("main"));
    assert_eq!(skills[1]["source"]["subpath"].as_str(), Some("."));
}

#[test]
fn add_fails_on_duplicate_id_and_does_not_modify_file() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");

    let init = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    assert_eq!(init.status.code(), Some(0));

    let before = fs::read_to_string(&config_path).expect("read config");

    let add = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args([
            "add",
            "--config",
            config_path.to_str().expect("utf8 path"),
            "--id",
            "browser-tool",
            "--repo",
            "https://example.com/repo.git",
            "--target",
            "claude-code",
        ])
        .output()
        .expect("run add duplicate");

    assert_eq!(
        add.status.code(),
        Some(2),
        "add duplicate should exit 2, stderr={}",
        String::from_utf8_lossy(&add.stderr)
    );

    let after = fs::read_to_string(&config_path).expect("read config");
    assert_eq!(before, after, "config must not be modified on failure");
}

#[test]
fn remove_deletes_only_matching_skill() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");

    let init = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    assert_eq!(init.status.code(), Some(0));

    let add = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args([
            "add",
            "--config",
            config_path.to_str().expect("utf8 path"),
            "--id",
            "new-skill",
            "--repo",
            "https://example.com/repo.git",
            "--target",
            "claude-code",
        ])
        .output()
        .expect("run add");
    assert_eq!(add.status.code(), Some(0));

    let remove = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args([
            "remove",
            "new-skill",
            "--config",
            config_path.to_str().expect("utf8 path"),
        ])
        .output()
        .expect("run remove");
    assert_eq!(
        remove.status.code(),
        Some(0),
        "remove should succeed, stderr={}",
        String::from_utf8_lossy(&remove.stderr)
    );

    let written = fs::read_to_string(&config_path).expect("read config");
    let value: toml::Value = toml::from_str(&written).expect("config should be valid toml");
    let skills = value
        .get("skills")
        .and_then(|v| v.as_array())
        .expect("skills should be an array");
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0]["id"].as_str(), Some("browser-tool"));
}

#[test]
fn set_requires_at_least_one_mutation_flag() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");

    let init = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    assert_eq!(init.status.code(), Some(0));

    let out = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args([
            "set",
            "browser-tool",
            "--config",
            config_path.to_str().expect("utf8 path"),
        ])
        .output()
        .expect("run set with no mutations");

    assert_eq!(
        out.status.code(),
        Some(2),
        "set with no mutation should exit 2, stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn set_updates_only_targeted_fields_and_validates_before_write() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");

    let init = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    assert_eq!(init.status.code(), Some(0));

    let add = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args([
            "add",
            "--config",
            config_path.to_str().expect("utf8 path"),
            "--id",
            "new-skill",
            "--repo",
            "https://example.com/repo.git",
            "--target",
            "claude-code",
        ])
        .output()
        .expect("run add");
    assert_eq!(add.status.code(), Some(0));

    let before = fs::read_to_string(&config_path).expect("read config");

    let set = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args([
            "set",
            "new-skill",
            "--config",
            config_path.to_str().expect("utf8 path"),
            "--repo",
            "https://example.com/other.git",
            "--mode",
            "copy",
            "--verify-check",
            "path-exists",
            "content-present",
        ])
        .output()
        .expect("run set");
    assert_eq!(
        set.status.code(),
        Some(0),
        "set should succeed, stderr={}",
        String::from_utf8_lossy(&set.stderr)
    );

    let written = fs::read_to_string(&config_path).expect("read config");
    let value: toml::Value = toml::from_str(&written).expect("config should be valid toml");
    let skills = value
        .get("skills")
        .and_then(|v| v.as_array())
        .expect("skills should be an array");
    assert_eq!(skills.len(), 2);

    let browser = &skills[0];
    assert_eq!(browser["id"].as_str(), Some("browser-tool"));
    assert_eq!(
        browser["source"]["repo"].as_str(),
        Some("https://github.com/vercel-labs/skills.git")
    );

    let updated = &skills[1];
    assert_eq!(updated["id"].as_str(), Some("new-skill"));
    assert_eq!(
        updated["source"]["repo"].as_str(),
        Some("https://example.com/other.git")
    );
    assert_eq!(updated["install"]["mode"].as_str(), Some("copy"));

    let set_invalid = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args([
            "set",
            "new-skill",
            "--config",
            config_path.to_str().expect("utf8 path"),
            "--repo",
            "http://invalid.example/repo.git",
        ])
        .output()
        .expect("run set invalid");
    assert_eq!(
        set_invalid.status.code(),
        Some(2),
        "set invalid should exit 2, stderr={}",
        String::from_utf8_lossy(&set_invalid.stderr)
    );

    let after_invalid = fs::read_to_string(&config_path).expect("read config after invalid");
    assert_ne!(
        before, after_invalid,
        "sanity: file should have changed after successful set"
    );
    assert_eq!(
        written, after_invalid,
        "config must not be modified on validation failure"
    );
}
