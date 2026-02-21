mod common;

use std::fs;
use std::path::Path;

use eden_skills_cli::commands::CommandOptions;
use eden_skills_core::lock::{lock_path_for_config, read_lock_file, LOCK_VERSION};

fn default_options() -> CommandOptions {
    CommandOptions {
        strict: false,
        json: false,
    }
}

// ---------------------------------------------------------------------------
// TM-P27-012: Lock init creates empty lock
// ---------------------------------------------------------------------------

#[test]
fn init_creates_empty_lock_file() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("skills.toml");

    eden_skills_cli::commands::init(config_path.to_str().unwrap(), false).unwrap();

    let lock_path = lock_path_for_config(&config_path);
    assert!(lock_path.exists(), "lock file should exist after init");

    let lock = read_lock_file(&lock_path).unwrap().unwrap();
    assert_eq!(lock.version, LOCK_VERSION);
    assert!(lock.skills.is_empty(), "init should produce empty lock");
}

#[test]
fn init_with_force_recreates_lock() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("skills.toml");

    eden_skills_cli::commands::init(config_path.to_str().unwrap(), false).unwrap();
    eden_skills_cli::commands::init(config_path.to_str().unwrap(), true).unwrap();

    let lock_path = lock_path_for_config(&config_path);
    let lock = read_lock_file(&lock_path).unwrap().unwrap();
    assert_eq!(lock.version, LOCK_VERSION);
    assert!(lock.skills.is_empty());
}

// ---------------------------------------------------------------------------
// TM-P27-008: Lock file co-location with custom config
// ---------------------------------------------------------------------------

#[test]
fn lock_co_located_with_custom_config_path() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("custom-skills.toml");

    eden_skills_cli::commands::init(config_path.to_str().unwrap(), false).unwrap();

    let expected_lock = dir.path().join("custom-skills.lock");
    assert!(
        expected_lock.exists(),
        "lock should be co-located at {}",
        expected_lock.display()
    );
}

// ---------------------------------------------------------------------------
// TM-P27-001: Lock file creation on first apply
// ---------------------------------------------------------------------------

#[tokio::test]
async fn apply_creates_lock_on_first_run() {
    let dir = tempfile::tempdir().unwrap();
    let storage = dir.path().join("storage");
    let target = dir.path().join("target");
    fs::create_dir_all(&target).unwrap();

    let origin = common::init_origin_repo(dir.path());
    let repo_url = common::as_file_url(&origin);
    let config_path = common::write_config(
        dir.path(),
        &repo_url,
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage,
        &target,
    );

    let lock_path = lock_path_for_config(&config_path);
    assert!(!lock_path.exists(), "lock should not exist before apply");

    eden_skills_cli::commands::apply_async(config_path.to_str().unwrap(), default_options(), None)
        .await
        .unwrap();

    assert!(lock_path.exists(), "lock should exist after apply");
    let lock = read_lock_file(&lock_path).unwrap().unwrap();
    assert_eq!(lock.version, LOCK_VERSION);
    assert_eq!(lock.skills.len(), 1);
    assert_eq!(lock.skills[0].id, common::SKILL_ID);
    assert_eq!(lock.skills[0].install_mode, "symlink");
    assert!(!lock.skills[0].installed_at.is_empty());
    assert!(!lock.skills[0].targets.is_empty());
}

// ---------------------------------------------------------------------------
// TM-P27-006: Missing lock file fallback
// ---------------------------------------------------------------------------

#[tokio::test]
async fn apply_succeeds_without_existing_lock_file() {
    let dir = tempfile::tempdir().unwrap();
    let storage = dir.path().join("storage");
    let target = dir.path().join("target");
    fs::create_dir_all(&target).unwrap();

    let origin = common::init_origin_repo(dir.path());
    let repo_url = common::as_file_url(&origin);
    let config_path = common::write_config(
        dir.path(),
        &repo_url,
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage,
        &target,
    );

    let lock_path = lock_path_for_config(&config_path);
    assert!(!lock_path.exists());

    let result = eden_skills_cli::commands::apply_async(
        config_path.to_str().unwrap(),
        default_options(),
        None,
    )
    .await;

    assert!(result.is_ok(), "apply should succeed without lock file");
    assert!(
        lock_path.exists(),
        "lock should be created after successful apply"
    );
}

// ---------------------------------------------------------------------------
// TM-P27-007: Corrupted lock file recovery
// ---------------------------------------------------------------------------

#[tokio::test]
async fn apply_recovers_from_corrupted_lock() {
    let dir = tempfile::tempdir().unwrap();
    let storage = dir.path().join("storage");
    let target = dir.path().join("target");
    fs::create_dir_all(&target).unwrap();

    let origin = common::init_origin_repo(dir.path());
    let repo_url = common::as_file_url(&origin);
    let config_path = common::write_config(
        dir.path(),
        &repo_url,
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage,
        &target,
    );

    let lock_path = lock_path_for_config(&config_path);
    fs::write(&lock_path, "GARBAGE CONTENT {{{{ NOT TOML").unwrap();

    let result = eden_skills_cli::commands::apply_async(
        config_path.to_str().unwrap(),
        default_options(),
        None,
    )
    .await;

    assert!(
        result.is_ok(),
        "apply should succeed despite corrupted lock"
    );
    let lock = read_lock_file(&lock_path).unwrap().unwrap();
    assert_eq!(lock.version, LOCK_VERSION);
    assert_eq!(lock.skills.len(), 1);
}

// ---------------------------------------------------------------------------
// TM-P27-002: Lock file updated after install
// ---------------------------------------------------------------------------

#[tokio::test]
async fn install_creates_lock_file() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("skills.toml");
    eden_skills_cli::commands::init(config_path.to_str().unwrap(), false).unwrap();

    let origin = common::init_origin_repo(dir.path());
    let local_path = origin.display().to_string();

    let result =
        eden_skills_cli::commands::install_async(eden_skills_cli::commands::InstallRequest {
            config_path: config_path.to_str().unwrap().to_string(),
            source: local_path,
            id: Some("installed-skill".to_string()),
            r#ref: None,
            skill: vec![],
            all: true,
            list: false,
            version: None,
            registry: None,
            target: vec!["custom:".to_string() + dir.path().join("tgt").to_str().unwrap()],
            dry_run: false,
            options: default_options(),
        })
        .await;

    assert!(result.is_ok(), "install should succeed: {:?}", result.err());

    let lock_path = lock_path_for_config(&config_path);
    assert!(lock_path.exists(), "lock should exist after install");
    let lock = read_lock_file(&lock_path).unwrap().unwrap();
    assert!(
        lock.skills.iter().any(|s| s.id == "installed-skill"),
        "lock should contain installed skill"
    );
}

// ---------------------------------------------------------------------------
// TM-P27-003: Lock file updated after remove
// ---------------------------------------------------------------------------

#[tokio::test]
async fn remove_updates_lock_file() {
    let dir = tempfile::tempdir().unwrap();
    let storage = dir.path().join("storage");
    let target = dir.path().join("target");
    fs::create_dir_all(&target).unwrap();

    let origin = common::init_origin_repo(dir.path());
    let repo_url = common::as_file_url(&origin);
    let config_path = common::write_config(
        dir.path(),
        &repo_url,
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage,
        &target,
    );

    eden_skills_cli::commands::apply_async(config_path.to_str().unwrap(), default_options(), None)
        .await
        .unwrap();

    let lock_path = lock_path_for_config(&config_path);
    let lock_before = read_lock_file(&lock_path).unwrap().unwrap();
    assert_eq!(lock_before.skills.len(), 1);

    eden_skills_cli::commands::remove_async(
        config_path.to_str().unwrap(),
        common::SKILL_ID,
        default_options(),
    )
    .await
    .unwrap();

    let lock_after = read_lock_file(&lock_path).unwrap().unwrap();
    assert!(
        lock_after.skills.is_empty(),
        "lock should have no skills after remove"
    );
}

// ---------------------------------------------------------------------------
// TM-P27-009: Lock entries sorted alphabetically
// ---------------------------------------------------------------------------

#[tokio::test]
async fn lock_entries_sorted_after_apply() {
    let dir = tempfile::tempdir().unwrap();
    let storage = dir.path().join("storage");
    let target_root = dir.path().join("target");
    fs::create_dir_all(&target_root).unwrap();

    let origin = common::init_origin_repo(dir.path());
    let repo_url = common::as_file_url(&origin);

    let toml_escape = |p: &Path| p.display().to_string().replace('\\', "\\\\");
    let config_content = format!(
        r#"version = 1

[storage]
root = "{}"

[[skills]]
id = "zebra-skill"

[skills.source]
repo = "{}"
subpath = "packages/browser"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "custom"
path = "{}"

[skills.verify]
enabled = true
checks = ["path-exists", "target-resolves", "is-symlink"]

[[skills]]
id = "alpha-skill"

[skills.source]
repo = "{}"
subpath = "packages/browser"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "custom"
path = "{}"

[skills.verify]
enabled = true
checks = ["path-exists", "target-resolves", "is-symlink"]
"#,
        toml_escape(&storage),
        repo_url.replace('\\', "\\\\").replace('"', "\\\""),
        toml_escape(&target_root),
        repo_url.replace('\\', "\\\\").replace('"', "\\\""),
        toml_escape(&target_root),
    );

    let config_path = dir.path().join("skills.toml");
    fs::write(&config_path, config_content).unwrap();

    eden_skills_cli::commands::apply_async(config_path.to_str().unwrap(), default_options(), None)
        .await
        .unwrap();

    let lock_path = lock_path_for_config(&config_path);
    let lock = read_lock_file(&lock_path).unwrap().unwrap();
    assert_eq!(lock.skills.len(), 2);

    let ids: Vec<&str> = lock.skills.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["alpha-skill", "zebra-skill"],
        "lock entries should be sorted alphabetically"
    );
}
