use std::fs;

use eden_skills_core::config::{load_from_file, LoadOptions};
use tempfile::tempdir;

#[test]
fn load_valid_config_with_defaults() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("skills.toml");
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

    let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
    assert_eq!(
        loaded.config.storage_root,
        "~/.local/share/eden-skills/repos"
    );
    assert_eq!(loaded.config.skills.len(), 1);
    assert_eq!(loaded.config.skills[0].source.subpath, ".");
    assert_eq!(loaded.config.skills[0].source.r#ref, "main");
    assert_eq!(
        loaded.config.skills[0].verify.checks,
        vec![
            "path-exists".to_string(),
            "target-resolves".to_string(),
            "is-symlink".to_string()
        ]
    );
}

#[test]
fn reject_custom_target_without_path() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[[skills]]
id = "x"

[skills.source]
repo = "https://github.com/vercel-labs/skills.git"

[[skills.targets]]
agent = "custom"
"#,
    )
    .expect("write config");

    let err = load_from_file(&config_path, LoadOptions::default()).expect_err("expected error");
    let message = err.to_string();
    assert!(message.contains("skills[0].targets[0].path"));
}
