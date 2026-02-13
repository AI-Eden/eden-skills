use std::env;
use std::path::{Path, PathBuf};

use eden_skills_core::config::{AgentKind, TargetConfig};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::{normalize_lexical, resolve_path_string, resolve_target_path};
use tempfile::tempdir;

#[test]
fn resolve_target_path_prefers_explicit_path_over_expected_path() {
    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();

    let target = TargetConfig {
        agent: AgentKind::ClaudeCode,
        expected_path: Some("/ignored/expected".to_string()),
        path: Some("./explicit/../final".to_string()),
    };

    let resolved = resolve_target_path(&target, config_dir).expect("resolve target path");
    assert_eq!(resolved, config_dir.join("final"));
}

#[test]
fn resolve_target_path_uses_expected_path_when_path_missing() {
    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();

    let target = TargetConfig {
        agent: AgentKind::Cursor,
        expected_path: Some("nested/./expected".to_string()),
        path: None,
    };

    let resolved = resolve_target_path(&target, config_dir).expect("resolve target path");
    assert_eq!(resolved, config_dir.join("nested").join("expected"));
}

#[test]
fn resolve_target_path_uses_default_agent_path_when_no_explicit_paths() {
    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();
    let home = env::var("HOME").expect("HOME must be set for tests");

    let target = TargetConfig {
        agent: AgentKind::ClaudeCode,
        expected_path: None,
        path: None,
    };

    let resolved = resolve_target_path(&target, config_dir).expect("resolve target path");
    assert_eq!(resolved, PathBuf::from(home).join(".claude").join("skills"));
}

#[test]
fn resolve_target_path_fails_for_custom_without_paths() {
    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();

    let target = TargetConfig {
        agent: AgentKind::Custom,
        expected_path: None,
        path: None,
    };

    let err = resolve_target_path(&target, config_dir).expect_err("expected error");
    assert!(matches!(err, EdenError::Validation(_)));
    assert!(err.to_string().contains("TARGET_PATH_UNRESOLVED"));
}

#[test]
fn resolve_path_string_expands_tilde_and_normalizes() {
    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();
    let home = env::var("HOME").expect("HOME must be set for tests");

    let resolved = resolve_path_string("~/a/./b/../c", config_dir).expect("resolve path string");
    assert_eq!(resolved, PathBuf::from(home).join("a").join("c"));
}

#[test]
fn resolve_path_string_resolves_relative_against_config_dir_and_normalizes() {
    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path().join("cfg");
    std::fs::create_dir_all(&config_dir).expect("create config dir");

    let resolved = resolve_path_string("a/./b/../c", &config_dir).expect("resolve path string");
    assert_eq!(resolved, config_dir.join("a").join("c"));
}

#[test]
fn resolve_path_string_rejects_unsupported_tilde_forms() {
    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();

    let err = resolve_path_string("~someone/dir", config_dir).expect_err("expected error");
    assert!(matches!(err, EdenError::Validation(_)));
    assert!(err.to_string().contains("unsupported home expansion"));
}

#[test]
fn normalize_lexical_collapses_dot_and_dotdot() {
    let path = Path::new("/tmp/a/./b/../c");
    assert_eq!(normalize_lexical(path), PathBuf::from("/tmp/a/c"));
}
