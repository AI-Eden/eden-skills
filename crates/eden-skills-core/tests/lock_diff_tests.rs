use std::path::Path;

use eden_skills_core::config::*;
use eden_skills_core::lock::*;

fn make_skill(id: &str, repo: &str, subpath: &str, ref_: &str, mode: InstallMode) -> SkillConfig {
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
            path: Some("/targets".to_string()),
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
            path: format!("/targets/{id}"),
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
// Unchanged skill (identical fields)
// ---------------------------------------------------------------------------

#[test]
fn unchanged_skill_classified_correctly() {
    let config = make_config(vec![make_skill(
        "a",
        "https://example.com/a.git",
        ".",
        "main",
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

    assert!(diff.removed.is_empty());
    assert_eq!(diff.statuses["a"], SkillDiffStatus::Unchanged);
}

// ---------------------------------------------------------------------------
// Changed skill (repo changed)
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
// Mixed scenario: added + removed + unchanged + changed
// ---------------------------------------------------------------------------

#[test]
fn mixed_diff_scenario() {
    let config = make_config(vec![
        make_skill(
            "keep",
            "https://example.com/keep.git",
            ".",
            "main",
            InstallMode::Symlink,
        ),
        make_skill(
            "change",
            "https://example.com/change.git",
            "sub",
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
        skills: vec![
            make_lock_entry(
                "keep",
                "https://example.com/keep.git",
                ".",
                "main",
                "symlink",
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

    let diff = compute_lock_diff(&config, &lock, Path::new("/")).unwrap();

    assert_eq!(diff.statuses["keep"], SkillDiffStatus::Unchanged);
    assert_eq!(diff.statuses["change"], SkillDiffStatus::Changed);
    assert_eq!(diff.statuses["new"], SkillDiffStatus::Added);
    assert_eq!(diff.removed.len(), 1);
    assert_eq!(diff.removed[0].id, "orphan");
}
