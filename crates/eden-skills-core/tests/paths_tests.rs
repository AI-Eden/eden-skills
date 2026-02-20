use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use eden_skills_core::config::{AgentKind, TargetConfig};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::{normalize_lexical, resolve_path_string, resolve_target_path};
use tempfile::tempdir;

struct EnvVarReset {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarReset {
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            original: env::var_os(key),
        }
    }
}

impl Drop for EnvVarReset {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => env::set_var(self.key, value),
            None => env::remove_var(self.key),
        }
    }
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn resolve_target_path_prefers_explicit_path_over_expected_path() {
    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();

    let target = TargetConfig {
        agent: AgentKind::ClaudeCode,
        expected_path: Some("/ignored/expected".to_string()),
        path: Some("./explicit/../final".to_string()),
        environment: "local".to_string(),
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
        environment: "local".to_string(),
    };

    let resolved = resolve_target_path(&target, config_dir).expect("resolve target path");
    assert_eq!(resolved, config_dir.join("nested").join("expected"));
}

#[test]
fn resolve_target_path_uses_default_agent_path_when_no_explicit_paths() {
    let _env_guard = env_lock().lock().expect("lock env");
    let _home_reset = EnvVarReset::capture("HOME");
    let _userprofile_reset = EnvVarReset::capture("USERPROFILE");
    let home = tempdir().expect("tempdir home");
    env::set_var("HOME", home.path());

    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();

    let target = TargetConfig {
        agent: AgentKind::ClaudeCode,
        expected_path: None,
        path: None,
        environment: "local".to_string(),
    };

    let resolved = resolve_target_path(&target, config_dir).expect("resolve target path");
    assert_eq!(resolved, home.path().join(".claude").join("skills"));
}

#[test]
fn resolve_target_path_fails_for_custom_without_paths() {
    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();

    let target = TargetConfig {
        agent: AgentKind::Custom,
        expected_path: None,
        path: None,
        environment: "local".to_string(),
    };

    let err = resolve_target_path(&target, config_dir).expect_err("expected error");
    assert!(matches!(err, EdenError::Validation(_)));
    assert!(err.to_string().contains("TARGET_PATH_UNRESOLVED"));
}

#[test]
fn resolve_target_path_supports_extended_agent_default_paths() {
    let _env_guard = env_lock().lock().expect("lock env");
    let _home_reset = EnvVarReset::capture("HOME");
    let _userprofile_reset = EnvVarReset::capture("USERPROFILE");
    let home = tempdir().expect("tempdir home");
    env::set_var("HOME", home.path());

    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();

    let opencode = TargetConfig {
        agent: AgentKind::Opencode,
        expected_path: None,
        path: None,
        environment: "local".to_string(),
    };
    let windsurf = TargetConfig {
        agent: AgentKind::Windsurf,
        expected_path: None,
        path: None,
        environment: "local".to_string(),
    };
    let adal = TargetConfig {
        agent: AgentKind::Adal,
        expected_path: None,
        path: None,
        environment: "local".to_string(),
    };

    assert_eq!(
        resolve_target_path(&opencode, config_dir).expect("resolve opencode"),
        home.path().join(".agents").join("skills")
    );
    assert_eq!(
        resolve_target_path(&windsurf, config_dir).expect("resolve windsurf"),
        home.path().join(".windsurf").join("skills")
    );
    assert_eq!(
        resolve_target_path(&adal, config_dir).expect("resolve adal"),
        home.path().join(".adal").join("skills")
    );
}

#[test]
fn resolve_path_string_expands_tilde_and_normalizes() {
    let _env_guard = env_lock().lock().expect("lock env");
    let _home_reset = EnvVarReset::capture("HOME");
    let home = tempdir().expect("tempdir home");
    env::set_var("HOME", home.path());

    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();

    let resolved = resolve_path_string("~/a/./b/../c", config_dir).expect("resolve path string");
    assert_eq!(resolved, home.path().join("a").join("c"));
}

#[test]
fn resolve_path_string_uses_userprofile_when_home_is_unset() {
    let _env_guard = env_lock().lock().expect("lock env");
    let _home_reset = EnvVarReset::capture("HOME");
    let _userprofile_reset = EnvVarReset::capture("USERPROFILE");
    env::remove_var("HOME");

    let userprofile = tempdir().expect("tempdir userprofile");
    env::set_var("USERPROFILE", userprofile.path());

    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();
    let resolved = resolve_path_string("~/agent/skills", config_dir).expect("resolve path string");
    assert_eq!(resolved, userprofile.path().join("agent").join("skills"));
}

#[test]
fn resolve_path_string_prefers_home_when_home_and_userprofile_exist() {
    let _env_guard = env_lock().lock().expect("lock env");
    let _home_reset = EnvVarReset::capture("HOME");
    let _userprofile_reset = EnvVarReset::capture("USERPROFILE");

    let home = tempdir().expect("tempdir home");
    let userprofile = tempdir().expect("tempdir userprofile");
    env::set_var("HOME", home.path());
    env::set_var("USERPROFILE", userprofile.path());

    let dir = tempdir().expect("tempdir");
    let config_dir = dir.path();
    let resolved = resolve_path_string("~/agent/skills", config_dir).expect("resolve path string");
    assert_eq!(resolved, home.path().join("agent").join("skills"));
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
    #[cfg(unix)]
    {
        let path = Path::new("/tmp/a/./b/../c");
        assert_eq!(normalize_lexical(path), PathBuf::from("/tmp/a/c"));
    }

    #[cfg(windows)]
    {
        let path = Path::new(r"C:\tmp\a\.\b\..\c");
        assert_eq!(normalize_lexical(path), PathBuf::from(r"C:\tmp\a\c"));
    }
}
