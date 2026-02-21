mod common;

use std::fs;

use eden_skills_cli::commands::CommandOptions;
use eden_skills_core::lock::{
    lock_path_for_config, read_lock_file, write_lock_file, LockFile, LockSkillEntry, LockTarget,
    LOCK_VERSION,
};

fn default_options() -> CommandOptions {
    CommandOptions {
        strict: false,
        json: false,
    }
}

fn toml_escape(p: &std::path::Path) -> String {
    p.display().to_string().replace('\\', "\\\\")
}

fn toml_escape_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ---------------------------------------------------------------------------
// TM-P27-004: Orphan removal via apply
// ---------------------------------------------------------------------------

#[tokio::test]
async fn apply_removes_orphaned_skill_from_lock() {
    let dir = tempfile::tempdir().unwrap();
    let storage = dir.path().join("storage");
    let target = dir.path().join("target");
    fs::create_dir_all(&target).unwrap();

    let origin = common::init_origin_repo(dir.path());
    let repo_url = common::as_file_url(&origin);

    let config_content = format!(
        r#"version = 1

[storage]
root = "{storage}"

[[skills]]
id = "skill-a"
[skills.source]
repo = "{repo}"
subpath = "packages/browser"
ref = "main"
[skills.install]
mode = "symlink"
[[skills.targets]]
agent = "custom"
path = "{target}"
[skills.verify]
enabled = true
checks = ["path-exists", "target-resolves", "is-symlink"]

[[skills]]
id = "skill-b"
[skills.source]
repo = "{repo}"
subpath = "packages/browser"
ref = "main"
[skills.install]
mode = "symlink"
[[skills.targets]]
agent = "custom"
path = "{target}"
[skills.verify]
enabled = true
checks = ["path-exists", "target-resolves", "is-symlink"]
"#,
        storage = toml_escape(&storage),
        repo = toml_escape_str(&repo_url),
        target = toml_escape(&target),
    );

    let config_path = dir.path().join("skills.toml");
    fs::write(&config_path, &config_content).unwrap();

    // First apply: install both skills.
    eden_skills_cli::commands::apply_async(config_path.to_str().unwrap(), default_options(), None)
        .await
        .unwrap();

    let lock_path = lock_path_for_config(&config_path);
    let lock_before = read_lock_file(&lock_path).unwrap().unwrap();
    assert_eq!(lock_before.skills.len(), 2);
    assert!(target.join("skill-a").exists() || target.join("skill-b").exists());

    // Remove skill-b from config, keeping only skill-a.
    let config_a_only = format!(
        r#"version = 1

[storage]
root = "{storage}"

[[skills]]
id = "skill-a"
[skills.source]
repo = "{repo}"
subpath = "packages/browser"
ref = "main"
[skills.install]
mode = "symlink"
[[skills.targets]]
agent = "custom"
path = "{target}"
[skills.verify]
enabled = true
checks = ["path-exists", "target-resolves", "is-symlink"]
"#,
        storage = toml_escape(&storage),
        repo = toml_escape_str(&repo_url),
        target = toml_escape(&target),
    );
    fs::write(&config_path, config_a_only).unwrap();

    // Second apply: should remove skill-b (orphan).
    eden_skills_cli::commands::apply_async(config_path.to_str().unwrap(), default_options(), None)
        .await
        .unwrap();

    let lock_after = read_lock_file(&lock_path).unwrap().unwrap();
    assert_eq!(lock_after.skills.len(), 1, "lock should have 1 skill");
    assert_eq!(lock_after.skills[0].id, "skill-a");

    assert!(
        !target.join("skill-b").exists(),
        "orphaned skill-b target should be cleaned up"
    );
    assert!(
        !storage.join("skill-b").exists(),
        "orphaned skill-b storage should be cleaned up"
    );
}

// ---------------------------------------------------------------------------
// TM-P27-005: Plan shows remove actions
// ---------------------------------------------------------------------------

#[test]
fn plan_shows_remove_actions_for_orphans() {
    let dir = tempfile::tempdir().unwrap();
    let storage = dir.path().join("storage");
    let target = dir.path().join("target");
    fs::create_dir_all(&target).unwrap();

    let config_content = format!(
        r#"version = 1

[storage]
root = "{}"
"#,
        toml_escape(&storage),
    );
    let config_path = dir.path().join("skills.toml");
    fs::write(&config_path, config_content).unwrap();

    let lock = LockFile {
        version: LOCK_VERSION,
        skills: vec![LockSkillEntry {
            id: "orphan-skill".to_string(),
            source_repo: "https://example.com/repo.git".to_string(),
            source_subpath: ".".to_string(),
            source_ref: "main".to_string(),
            resolved_commit: "abc123".to_string(),
            resolved_version: None,
            install_mode: "symlink".to_string(),
            installed_at: "2026-02-21T10:00:00Z".to_string(),
            targets: vec![LockTarget {
                agent: "claude-code".to_string(),
                path: target.join("orphan-skill").display().to_string(),
            }],
        }],
    };
    let lock_path = lock_path_for_config(&config_path);
    write_lock_file(&lock_path, &lock).unwrap();

    let result = eden_skills_cli::commands::plan(config_path.to_str().unwrap(), default_options());
    assert!(result.is_ok());
}

#[test]
fn plan_json_includes_remove_action() {
    let dir = tempfile::tempdir().unwrap();
    let storage = dir.path().join("storage");

    let config_content = format!(
        r#"version = 1

[storage]
root = "{}"
"#,
        toml_escape(&storage),
    );
    let config_path = dir.path().join("skills.toml");
    fs::write(&config_path, config_content).unwrap();

    let lock = LockFile {
        version: LOCK_VERSION,
        skills: vec![LockSkillEntry {
            id: "orphan".to_string(),
            source_repo: "https://example.com/repo.git".to_string(),
            source_subpath: ".".to_string(),
            source_ref: "main".to_string(),
            resolved_commit: "".to_string(),
            resolved_version: None,
            install_mode: "symlink".to_string(),
            installed_at: "2026-02-21T10:00:00Z".to_string(),
            targets: vec![LockTarget {
                agent: "cursor".to_string(),
                path: "/tmp/orphan".to_string(),
            }],
        }],
    };
    let lock_path = lock_path_for_config(&config_path);
    write_lock_file(&lock_path, &lock).unwrap();

    let result = eden_skills_cli::commands::plan(
        config_path.to_str().unwrap(),
        CommandOptions {
            strict: false,
            json: true,
        },
    );
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// TM-P27-010: Lock preserves resolved commit
// ---------------------------------------------------------------------------

#[tokio::test]
async fn lock_records_resolved_commit_after_apply() {
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
    let lock = read_lock_file(&lock_path).unwrap().unwrap();
    assert_eq!(lock.skills.len(), 1);

    let commit = &lock.skills[0].resolved_commit;
    assert!(
        commit.len() == 40,
        "resolved_commit should be 40-char hex SHA, got: '{commit}'"
    );
    assert!(
        commit.chars().all(|c| c.is_ascii_hexdigit()),
        "resolved_commit should be hex: '{commit}'"
    );
}

// ---------------------------------------------------------------------------
// TM-P27-011: Apply noop optimization
// ---------------------------------------------------------------------------

#[tokio::test]
async fn apply_noop_with_unchanged_config() {
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

    // First apply
    eden_skills_cli::commands::apply_async(config_path.to_str().unwrap(), default_options(), None)
        .await
        .unwrap();

    // Second apply with identical config — should succeed with noop
    let result = eden_skills_cli::commands::apply_async(
        config_path.to_str().unwrap(),
        default_options(),
        None,
    )
    .await;
    assert!(
        result.is_ok(),
        "second apply should succeed: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// TM-P27-015: Strict mode does not block removals
// ---------------------------------------------------------------------------

#[tokio::test]
async fn strict_mode_does_not_block_removals() {
    let dir = tempfile::tempdir().unwrap();
    let storage = dir.path().join("storage");
    let target = dir.path().join("target");
    fs::create_dir_all(&target).unwrap();

    let origin = common::init_origin_repo(dir.path());
    let repo_url = common::as_file_url(&origin);

    let config_content = format!(
        r#"version = 1

[storage]
root = "{storage}"

[[skills]]
id = "to-remove"
[skills.source]
repo = "{repo}"
subpath = "packages/browser"
ref = "main"
[skills.install]
mode = "symlink"
[[skills.targets]]
agent = "custom"
path = "{target}"
[skills.verify]
enabled = true
checks = ["path-exists", "target-resolves", "is-symlink"]
"#,
        storage = toml_escape(&storage),
        repo = toml_escape_str(&repo_url),
        target = toml_escape(&target),
    );

    let config_path = dir.path().join("skills.toml");
    fs::write(&config_path, &config_content).unwrap();

    // Apply to install the skill
    eden_skills_cli::commands::apply_async(config_path.to_str().unwrap(), default_options(), None)
        .await
        .unwrap();

    // Remove skill from config
    let empty_config = format!(
        r#"version = 1

[storage]
root = "{}"
"#,
        toml_escape(&storage),
    );
    fs::write(&config_path, empty_config).unwrap();

    // Apply with --strict should succeed (removals are not conflicts)
    let result = eden_skills_cli::commands::apply_async(
        config_path.to_str().unwrap(),
        CommandOptions {
            strict: true,
            json: false,
        },
        None,
    )
    .await;

    assert!(
        result.is_ok(),
        "strict apply should succeed for removals: {:?}",
        result.err()
    );

    let lock_path = lock_path_for_config(&config_path);
    let lock = read_lock_file(&lock_path).unwrap().unwrap();
    assert!(
        lock.skills.is_empty(),
        "removed skill should be gone from lock"
    );
}
