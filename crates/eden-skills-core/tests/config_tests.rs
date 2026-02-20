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

#[test]
fn repo_url_allows_https_ssh_scp_like_and_file() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("skills.toml");

    for url in [
        "https://example.com/repo.git",
        "ssh://example.com/repo.git",
        "git@example.com:org/repo.git",
        "file:///tmp/repo.git",
    ] {
        fs::write(
            &config_path,
            format!(
                r#"
version = 1

[[skills]]
id = "x"

[skills.source]
repo = "{url}"

[[skills.targets]]
agent = "claude-code"
"#
            ),
        )
        .expect("write config");

        load_from_file(&config_path, LoadOptions::default())
            .unwrap_or_else(|err| panic!("expected url `{url}` to be valid, err={err}"));
    }
}

#[test]
fn repo_url_rejects_non_git_schemes() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[[skills]]
id = "x"

[skills.source]
repo = "http://example.com/repo.git"

[[skills.targets]]
agent = "claude-code"
"#,
    )
    .expect("write config");

    let err = load_from_file(&config_path, LoadOptions::default()).expect_err("expected error");
    let message = err.to_string();
    assert!(
        message.contains("skills[0].source.repo"),
        "error should reference field path, got `{message}`"
    );
}

#[test]
fn load_valid_config_when_skills_array_is_missing() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[storage]
root = "./storage"
"#,
    )
    .expect("write config");

    let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
    assert_eq!(loaded.config.version, 1);
    assert_eq!(loaded.config.skills.len(), 0);
}

#[test]
fn load_valid_config_when_skills_array_is_explicitly_empty() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[storage]
root = "./storage"

skills = []
"#,
    )
    .expect("write config");

    let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
    assert_eq!(loaded.config.version, 1);
    assert_eq!(loaded.config.skills.len(), 0);
}

#[test]
fn load_phase1_style_config_with_five_skills_for_backward_compatibility() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("skills.toml");

    let mut content = String::from("version = 1\n\n[storage]\nroot = \"./storage\"\n\n");
    for idx in 0..5 {
        content.push_str(&format!(
            "[[skills]]\nid = \"skill-{idx}\"\n\n[skills.source]\nrepo = \"https://github.com/example/repo-{idx}.git\"\nsubpath = \".\"\nref = \"main\"\n\n[[skills.targets]]\nagent = \"claude-code\"\n\n"
        ));
    }
    fs::write(&config_path, content).expect("write config");

    let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
    assert_eq!(loaded.config.skills.len(), 5);
    assert_eq!(loaded.config.skills[0].id, "skill-0");
    assert_eq!(loaded.config.skills[4].id, "skill-4");
}

#[test]
fn load_valid_config_with_extended_agent_aliases() {
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
agent = "opencode"

[[skills.targets]]
agent = "windsurf"
"#,
    )
    .expect("write config");

    let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
    assert_eq!(loaded.config.skills.len(), 1);
    assert_eq!(loaded.config.skills[0].targets.len(), 2);
}
