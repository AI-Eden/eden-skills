use std::fs;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn init_creates_config_when_missing() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");

    assert_eq!(
        output.status.code(),
        Some(0),
        "init should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&config_path).expect("read config");
    assert!(content.contains("version = 1"));
    assert!(content.contains("[[skills]]"));
}

#[test]
fn init_fails_when_config_exists_without_force() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");
    fs::write(&config_path, "version = 1\n").expect("seed file");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");

    assert_eq!(
        output.status.code(),
        Some(3),
        "init should fail with conflict exit code 3, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn init_overwrites_when_force_is_set() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");
    fs::write(&config_path, "version = 1\n# old\n").expect("seed file");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["init", "--force", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init --force");

    assert_eq!(
        output.status.code(),
        Some(0),
        "init --force should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&config_path).expect("read config");
    assert!(content.contains("browser-tool"));
    assert!(!content.contains("# old"));
}

#[test]
fn init_supports_tilde_in_config_path() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path();
    let config_path = home.join(".config").join("eden-skills").join("skills.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .env("HOME", home)
        .args(["init", "--config", "~/.config/eden-skills/skills.toml"])
        .output()
        .expect("run init with tilde");

    assert_eq!(
        output.status.code(),
        Some(0),
        "init with tilde should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        config_path.exists(),
        "expected config to be written under HOME"
    );
}
