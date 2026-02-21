use std::fs;
use std::path::Path;

use eden_skills_core::lock::{
    lock_path_for_config, read_lock_file, write_lock_file, LockFile, LockSkillEntry, LockTarget,
    LOCK_VERSION,
};

// ---------------------------------------------------------------------------
// LCK-004: lock_path_for_config
// ---------------------------------------------------------------------------

#[test]
fn lock_path_replaces_toml_extension() {
    let config = Path::new("/home/user/.eden-skills/skills.toml");
    let lock = lock_path_for_config(config);
    assert_eq!(lock, Path::new("/home/user/.eden-skills/skills.lock"));
}

#[test]
fn lock_path_replaces_custom_name_toml() {
    let config = Path::new("/tmp/test-skills.toml");
    let lock = lock_path_for_config(config);
    assert_eq!(lock, Path::new("/tmp/test-skills.lock"));
}

#[test]
fn lock_path_appends_lock_when_no_toml_extension() {
    let config = Path::new("/tmp/myconfig");
    let lock = lock_path_for_config(config);
    assert_eq!(lock, Path::new("/tmp/myconfig.lock"));
}

#[test]
fn lock_path_appends_lock_for_yaml_extension() {
    let config = Path::new("/tmp/skills.yaml");
    let lock = lock_path_for_config(config);
    assert_eq!(lock, Path::new("/tmp/skills.yaml.lock"));
}

// ---------------------------------------------------------------------------
// LCK-003: Lock file TOML format round-trip
// ---------------------------------------------------------------------------

#[test]
fn lock_file_round_trip_serialization() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");

    let lock = LockFile {
        version: LOCK_VERSION,
        skills: vec![LockSkillEntry {
            id: "browser-tool".to_string(),
            source_repo: "https://github.com/vercel-labs/agent-skills.git".to_string(),
            source_subpath: "skills/browser-tool".to_string(),
            source_ref: "main".to_string(),
            resolved_commit: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
            resolved_version: None,
            install_mode: "symlink".to_string(),
            installed_at: "2026-02-21T10:30:00Z".to_string(),
            targets: vec![
                LockTarget {
                    agent: "claude-code".to_string(),
                    path: "~/.claude/skills/browser-tool".to_string(),
                },
                LockTarget {
                    agent: "cursor".to_string(),
                    path: "~/.cursor/skills/browser-tool".to_string(),
                },
            ],
        }],
    };

    write_lock_file(&lock_path, &lock).unwrap();
    let read_back = read_lock_file(&lock_path).unwrap().unwrap();

    assert_eq!(read_back.version, LOCK_VERSION);
    assert_eq!(read_back.skills.len(), 1);
    assert_eq!(read_back.skills[0].id, "browser-tool");
    assert_eq!(
        read_back.skills[0].resolved_commit,
        "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
    );
    assert_eq!(read_back.skills[0].targets.len(), 2);
}

#[test]
fn lock_file_contains_all_required_fields() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");

    let lock = LockFile {
        version: LOCK_VERSION,
        skills: vec![LockSkillEntry {
            id: "test-skill".to_string(),
            source_repo: "https://github.com/test/repo.git".to_string(),
            source_subpath: ".".to_string(),
            source_ref: "main".to_string(),
            resolved_commit: "".to_string(),
            resolved_version: Some("1.2.0".to_string()),
            install_mode: "copy".to_string(),
            installed_at: "2026-02-21T10:30:05Z".to_string(),
            targets: vec![LockTarget {
                agent: "claude-code".to_string(),
                path: "~/.claude/skills/test-skill".to_string(),
            }],
        }],
    };

    write_lock_file(&lock_path, &lock).unwrap();
    let content = fs::read_to_string(&lock_path).unwrap();

    assert!(content.contains("version = 1"));
    assert!(content.contains("id = \"test-skill\""));
    assert!(content.contains("source_repo = "));
    assert!(content.contains("source_subpath = "));
    assert!(content.contains("source_ref = "));
    assert!(content.contains("install_mode = "));
    assert!(content.contains("installed_at = "));
    assert!(content.contains("resolved_version = "));
    assert!(content.contains("agent = "));
    assert!(content.contains("path = "));
}

#[test]
fn lock_file_omits_resolved_version_when_none() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");

    let lock = LockFile {
        version: LOCK_VERSION,
        skills: vec![LockSkillEntry {
            id: "url-skill".to_string(),
            source_repo: "https://github.com/test/repo.git".to_string(),
            source_subpath: ".".to_string(),
            source_ref: "main".to_string(),
            resolved_commit: "".to_string(),
            resolved_version: None,
            install_mode: "symlink".to_string(),
            installed_at: "2026-02-21T10:30:00Z".to_string(),
            targets: vec![LockTarget {
                agent: "cursor".to_string(),
                path: "~/.cursor/skills/url-skill".to_string(),
            }],
        }],
    };

    write_lock_file(&lock_path, &lock).unwrap();
    let content = fs::read_to_string(&lock_path).unwrap();
    assert!(!content.contains("resolved_version"));
}

// ---------------------------------------------------------------------------
// LCK-003 / empty lock
// ---------------------------------------------------------------------------

#[test]
fn empty_lock_file_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");

    let lock = LockFile::empty();
    write_lock_file(&lock_path, &lock).unwrap();
    let read_back = read_lock_file(&lock_path).unwrap().unwrap();

    assert_eq!(read_back.version, LOCK_VERSION);
    assert!(read_back.skills.is_empty());
}

// ---------------------------------------------------------------------------
// LCK-009: Alphabetical sorting
// ---------------------------------------------------------------------------

#[test]
fn lock_entries_sorted_alphabetically_by_id() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");

    let lock = LockFile {
        version: LOCK_VERSION,
        skills: vec![
            make_entry("zebra"),
            make_entry("alpha"),
            make_entry("middle"),
        ],
    };

    write_lock_file(&lock_path, &lock).unwrap();
    let read_back = read_lock_file(&lock_path).unwrap().unwrap();

    let ids: Vec<&str> = read_back.skills.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["alpha", "middle", "zebra"]);
}

#[test]
fn lock_targets_sorted_alphabetically_by_agent() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");

    let lock = LockFile {
        version: LOCK_VERSION,
        skills: vec![LockSkillEntry {
            id: "skill-a".to_string(),
            source_repo: "https://example.com/repo.git".to_string(),
            source_subpath: ".".to_string(),
            source_ref: "main".to_string(),
            resolved_commit: "".to_string(),
            resolved_version: None,
            install_mode: "symlink".to_string(),
            installed_at: "2026-02-21T10:30:00Z".to_string(),
            targets: vec![
                LockTarget {
                    agent: "cursor".to_string(),
                    path: "/cursor/skill-a".to_string(),
                },
                LockTarget {
                    agent: "claude-code".to_string(),
                    path: "/claude/skill-a".to_string(),
                },
            ],
        }],
    };

    write_lock_file(&lock_path, &lock).unwrap();
    let read_back = read_lock_file(&lock_path).unwrap().unwrap();

    let agents: Vec<&str> = read_back.skills[0]
        .targets
        .iter()
        .map(|t| t.agent.as_str())
        .collect();
    assert_eq!(agents, vec!["claude-code", "cursor"]);
}

// ---------------------------------------------------------------------------
// LCK-005: Missing lock file graceful fallback
// ---------------------------------------------------------------------------

#[test]
fn missing_lock_file_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("nonexistent.lock");

    let result = read_lock_file(&lock_path).unwrap();
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// LCK-006: Corrupted lock file warning and recovery
// ---------------------------------------------------------------------------

#[test]
fn corrupted_lock_file_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");

    fs::write(&lock_path, "this is not valid TOML {{{{").unwrap();
    let result = read_lock_file(&lock_path).unwrap();
    assert!(result.is_none());
}

#[test]
fn unsupported_version_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");

    fs::write(&lock_path, "version = 999\n").unwrap();
    let result = read_lock_file(&lock_path).unwrap();
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// Internal timestamp formatter
// ---------------------------------------------------------------------------

#[test]
fn timestamp_format_is_iso8601() {
    // Verify the format convention used by make_entry helper.
    let entry = make_entry("test");
    assert!(
        entry.installed_at.ends_with('Z'),
        "timestamp should end with Z"
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_entry(id: &str) -> LockSkillEntry {
    LockSkillEntry {
        id: id.to_string(),
        source_repo: "https://example.com/repo.git".to_string(),
        source_subpath: ".".to_string(),
        source_ref: "main".to_string(),
        resolved_commit: "".to_string(),
        resolved_version: None,
        install_mode: "symlink".to_string(),
        installed_at: "2026-02-21T10:30:00Z".to_string(),
        targets: vec![LockTarget {
            agent: "claude-code".to_string(),
            path: format!("/skills/{id}"),
        }],
    }
}
