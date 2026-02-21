use std::path::Path;

use eden_skills_core::config::*;
use eden_skills_core::lock::*;

fn make_skill(id: &str, repo: &str, subpath: &str, ref_: &str, mode: InstallMode) -> SkillConfig {
    make_skill_with_target(id, repo, subpath, ref_, mode, "/targets")
}

fn make_skill_with_target(
    id: &str,
    repo: &str,
    subpath: &str,
    ref_: &str,
    mode: InstallMode,
    target_path: &str,
) -> SkillConfig {
    SkillConfig {
        id: id.to_string(),
        source: SourceConfig {
            repo: repo.to_string(),
            subpath: subpath.to_string(),
            r#ref: ref_.to_string(),
        },
        install: InstallConfig { mode },
        targets: vec![TargetConfig {
            agent: AgentKind::Custom,
            expected_path: None,
            path: Some(target_path.to_string()),
            environment: "local".to_string(),
        }],
        verify: VerifyConfig {
            enabled: false,
            checks: vec![],
        },
        safety: SafetyConfig {
            no_exec_metadata_only: false,
        },
    }
}

fn make_lock_entry(id: &str, repo: &str, subpath: &str, ref_: &str, mode: &str) -> LockSkillEntry {
    make_lock_entry_with_target(id, repo, subpath, ref_, mode, &format!("/targets/{id}"))
}

fn make_lock_entry_with_target(
    id: &str,
    repo: &str,
    subpath: &str,
    ref_: &str,
    mode: &str,
    resolved_target: &str,
) -> LockSkillEntry {
    LockSkillEntry {
        id: id.to_string(),
        source_repo: repo.to_string(),
        source_subpath: subpath.to_string(),
        source_ref: ref_.to_string(),
        resolved_commit: "abc123".to_string(),
        resolved_version: None,
        install_mode: mode.to_string(),
        installed_at: "2026-02-21T10:00:00Z".to_string(),
        targets: vec![LockTarget {
            agent: "custom".to_string(),
            path: resolved_target.to_string(),
        }],
    }
}

fn make_config(skills: Vec<SkillConfig>) -> Config {
    Config {
        version: 1,
        storage_root: "/storage".to_string(),
        reactor: ReactorConfig::default(),
        skills,
    }
}

// ---------------------------------------------------------------------------
// No lock file → all skills are Added, no removals
// ---------------------------------------------------------------------------

#[test]
fn no_lock_classifies_all_as_added() {
    let config = make_config(vec![
        make_skill(
            "a",
            "https://example.com/a.git",
            ".",
            "main",
            InstallMode::Symlink,
        ),
        make_skill(
            "b",
            "https://example.com/b.git",
            ".",
            "main",
            InstallMode::Symlink,
        ),
    ]);

    let diff = compute_lock_diff(&config, &None, Path::new("/")).unwrap();

    assert!(diff.removed.is_empty());
    assert_eq!(diff.statuses.len(), 2);
    assert_eq!(diff.statuses["a"], SkillDiffStatus::Added);
    assert_eq!(diff.statuses["b"], SkillDiffStatus::Added);
}

// ---------------------------------------------------------------------------
// Skill in lock but not in TOML → REMOVED
// ---------------------------------------------------------------------------

#[test]
fn skill_removed_from_toml_is_in_removed_set() {
    let config = make_config(vec![make_skill(
        "a",
        "https://example.com/a.git",
        ".",
        "main",
        InstallMode::Symlink,
    )]);

    let lock = Some(LockFile {
        version: LOCK_VERSION,
        skills: vec![
            make_lock_entry("a", "https://example.com/a.git", ".", "main", "symlink"),
            make_lock_entry("b", "https://example.com/b.git", ".", "main", "symlink"),
        ],
    });

    let diff = compute_lock_diff(&config, &lock, Path::new("/")).unwrap();

    assert_eq!(diff.removed.len(), 1);
    assert_eq!(diff.removed[0].id, "b");
    assert_eq!(diff.statuses.len(), 1);
}

// ---------------------------------------------------------------------------
// Skill in TOML but not in lock → ADDED
// ---------------------------------------------------------------------------

#[test]
fn new_skill_in_toml_is_added() {
    let config = make_config(vec![
        make_skill(
            "a",
            "https://example.com/a.git",
            ".",
            "main",
            InstallMode::Symlink,
        ),
        make_skill(
            "new",
            "https://example.com/new.git",
            ".",
            "main",
            InstallMode::Symlink,
        ),
    ]);

    let lock = Some(LockFile {
        version: LOCK_VERSION,
        skills: vec![make_lock_entry(
            "a",
            "https://example.com/a.git",
            ".",
            "main",
            "symlink",
        )],
    });

    let diff = compute_lock_diff(&config, &lock, Path::new("/")).unwrap();

    assert!(diff.removed.is_empty());
    assert_eq!(diff.statuses["new"], SkillDiffStatus::Added);
}

// ---------------------------------------------------------------------------
// Unchanged skill — uses real temp paths for cross-platform correctness
// ---------------------------------------------------------------------------

#[test]
fn unchanged_skill_classified_correctly() {
    let dir = tempfile::tempdir().unwrap();
    let target_dir = dir.path().join("targets");
    let target_str = target_dir.display().to_string();
    let resolved = target_dir.join("a").display().to_string();

    let config = make_config(vec![make_skill_with_target(
        "a",
        "https://example.com/a.git",
        ".",
        "main",
        InstallMode::Symlink,
        &target_str,
    )]);

    let lock = Some(LockFile {
        version: LOCK_VERSION,
        skills: vec![make_lock_entry_with_target(
            "a",
            "https://example.com/a.git",
            ".",
            "main",
            "symlink",
            &resolved,
        )],
    });

    let diff = compute_lock_diff(&config, &lock, dir.path()).unwrap();

    assert!(diff.removed.is_empty());
    assert_eq!(diff.statuses["a"], SkillDiffStatus::Unchanged);
}

// ---------------------------------------------------------------------------
// Changed skill (repo changed) — targets don't matter, short-circuits early
// ---------------------------------------------------------------------------

#[test]
fn changed_repo_classified_as_changed() {
    let config = make_config(vec![make_skill(
        "a",
        "https://example.com/new-repo.git",
        ".",
        "main",
        InstallMode::Symlink,
    )]);

    let lock = Some(LockFile {
        version: LOCK_VERSION,
        skills: vec![make_lock_entry(
            "a",
            "https://example.com/old-repo.git",
            ".",
            "main",
            "symlink",
        )],
    });

    let diff = compute_lock_diff(&config, &lock, Path::new("/")).unwrap();
    assert_eq!(diff.statuses["a"], SkillDiffStatus::Changed);
}

#[test]
fn changed_ref_classified_as_changed() {
    let config = make_config(vec![make_skill(
        "a",
        "https://example.com/a.git",
        ".",
        "v2.0",
        InstallMode::Symlink,
    )]);

    let lock = Some(LockFile {
        version: LOCK_VERSION,
        skills: vec![make_lock_entry(
            "a",
            "https://example.com/a.git",
            ".",
            "main",
            "symlink",
        )],
    });

    let diff = compute_lock_diff(&config, &lock, Path::new("/")).unwrap();
    assert_eq!(diff.statuses["a"], SkillDiffStatus::Changed);
}

#[test]
fn changed_install_mode_classified_as_changed() {
    let config = make_config(vec![make_skill(
        "a",
        "https://example.com/a.git",
        ".",
        "main",
        InstallMode::Copy,
    )]);

    let lock = Some(LockFile {
        version: LOCK_VERSION,
        skills: vec![make_lock_entry(
            "a",
            "https://example.com/a.git",
            ".",
            "main",
            "symlink",
        )],
    });

    let diff = compute_lock_diff(&config, &lock, Path::new("/")).unwrap();
    assert_eq!(diff.statuses["a"], SkillDiffStatus::Changed);
}

// ---------------------------------------------------------------------------
// Mixed scenario — uses real temp paths for "keep" (Unchanged) comparison
// ---------------------------------------------------------------------------

#[test]
fn mixed_diff_scenario() {
    let dir = tempfile::tempdir().unwrap();
    let target_dir = dir.path().join("targets");
    let target_str = target_dir.display().to_string();
    let keep_resolved = target_dir.join("keep").display().to_string();

    let config = make_config(vec![
        make_skill_with_target(
            "keep",
            "https://example.com/keep.git",
            ".",
            "main",
            InstallMode::Symlink,
            &target_str,
        ),
        make_skill_with_target(
            "change",
            "https://example.com/change.git",
            "sub",
            "main",
            InstallMode::Symlink,
            &target_str,
        ),
        make_skill_with_target(
            "new",
            "https://example.com/new.git",
            ".",
            "main",
            InstallMode::Symlink,
            &target_str,
        ),
    ]);

    let lock = Some(LockFile {
        version: LOCK_VERSION,
        skills: vec![
            make_lock_entry_with_target(
                "keep",
                "https://example.com/keep.git",
                ".",
                "main",
                "symlink",
                &keep_resolved,
            ),
            make_lock_entry(
                "change",
                "https://example.com/change.git",
                ".",
                "main",
                "symlink",
            ),
            make_lock_entry(
                "orphan",
                "https://example.com/orphan.git",
                ".",
                "main",
                "symlink",
            ),
        ],
    });

    let diff = compute_lock_diff(&config, &lock, dir.path()).unwrap();

    assert_eq!(diff.statuses["keep"], SkillDiffStatus::Unchanged);
    assert_eq!(diff.statuses["change"], SkillDiffStatus::Changed);
    assert_eq!(diff.statuses["new"], SkillDiffStatus::Added);
    assert_eq!(diff.removed.len(), 1);
    assert_eq!(diff.removed[0].id, "orphan");
}
