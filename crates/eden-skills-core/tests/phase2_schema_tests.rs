use std::io::Write;

use eden_skills_core::config::{load_from_file, LoadOptions};
use tempfile::NamedTempFile;

fn write_config(contents: &str) -> std::path::PathBuf {
    let mut file = NamedTempFile::new().expect("temp file");
    file.write_all(contents.as_bytes()).expect("write config");
    file.into_temp_path()
        .keep()
        .expect("persist temp config path")
}

#[test]
fn mode_b_skill_with_registries_is_accepted() {
    let path = write_config(
        r#"
version = 1

[registries]
official = { url = "https://example.com/official.git", priority = 100 }

[[skills]]
name = "google-search"
version = "^2.0"
registry = "official"

[[skills.targets]]
agent = "custom"
path = "/tmp/agent-skills"
"#,
    );

    let loaded = load_from_file(&path, LoadOptions::default()).expect("mode b config should load");
    assert_eq!(loaded.config.skills.len(), 1);
    assert_eq!(loaded.config.skills[0].id, "google-search");
}

#[test]
fn mode_b_without_registries_returns_missing_registries_code() {
    let path = write_config(
        r#"
version = 1

[[skills]]
name = "google-search"
version = "^2.0"

[[skills.targets]]
agent = "custom"
path = "/tmp/agent-skills"
"#,
    );

    let err = load_from_file(&path, LoadOptions::default()).expect_err("expected validation error");
    assert!(
        err.to_string().contains("MISSING_REGISTRIES"),
        "expected MISSING_REGISTRIES code, got {err}"
    );
}

#[test]
fn mixed_mode_fields_return_invalid_skill_mode_code() {
    let path = write_config(
        r#"
version = 1

[registries]
official = { url = "https://example.com/official.git", priority = 100 }

[[skills]]
id = "google-search"
name = "google-search"

[skills.source]
repo = "https://example.com/google-search.git"

[[skills.targets]]
agent = "custom"
path = "/tmp/agent-skills"
"#,
    );

    let err = load_from_file(&path, LoadOptions::default()).expect_err("expected validation error");
    assert!(
        err.to_string().contains("INVALID_SKILL_MODE"),
        "expected INVALID_SKILL_MODE code, got {err}"
    );
}

#[test]
fn unknown_registry_reference_returns_unknown_registry_code() {
    let path = write_config(
        r#"
version = 1

[registries]
official = { url = "https://example.com/official.git", priority = 100 }

[[skills]]
name = "google-search"
version = "^2.0"
registry = "forge"

[[skills.targets]]
agent = "custom"
path = "/tmp/agent-skills"
"#,
    );

    let err = load_from_file(&path, LoadOptions::default()).expect_err("expected validation error");
    assert!(
        err.to_string().contains("UNKNOWN_REGISTRY"),
        "expected UNKNOWN_REGISTRY code, got {err}"
    );
}

#[test]
fn invalid_semver_constraint_returns_invalid_semver_code() {
    let path = write_config(
        r#"
version = 1

[registries]
official = { url = "https://example.com/official.git", priority = 100 }

[[skills]]
name = "google-search"
version = "not-a-semver-constraint"

[[skills.targets]]
agent = "custom"
path = "/tmp/agent-skills"
"#,
    );

    let err = load_from_file(&path, LoadOptions::default()).expect_err("expected validation error");
    assert!(
        err.to_string().contains("INVALID_SEMVER"),
        "expected INVALID_SEMVER code, got {err}"
    );
}

#[test]
fn invalid_target_environment_returns_invalid_environment_code() {
    let path = write_config(
        r#"
version = 1

[[skills]]
id = "demo"

[skills.source]
repo = "https://example.com/demo.git"

[[skills.targets]]
agent = "custom"
path = "/tmp/agent-skills"
environment = "podman:demo"
"#,
    );

    let err = load_from_file(&path, LoadOptions::default()).expect_err("expected validation error");
    assert!(
        err.to_string().contains("INVALID_ENVIRONMENT"),
        "expected INVALID_ENVIRONMENT code, got {err}"
    );
}

#[test]
fn mode_a_mode_b_identifier_collision_returns_duplicate_skill_id_code() {
    let path = write_config(
        r#"
version = 1

[registries]
official = { url = "https://example.com/official.git", priority = 100 }

[[skills]]
id = "dup"

[skills.source]
repo = "https://example.com/direct.git"

[[skills.targets]]
agent = "custom"
path = "/tmp/agent-skills"

[[skills]]
name = "dup"
version = "^1.0"

[[skills.targets]]
agent = "custom"
path = "/tmp/agent-skills"
"#,
    );

    let err = load_from_file(&path, LoadOptions::default()).expect_err("expected validation error");
    assert!(
        err.to_string().contains("DUPLICATE_SKILL_ID"),
        "expected DUPLICATE_SKILL_ID code, got {err}"
    );
}

#[test]
fn reactor_concurrency_is_loaded_when_valid() {
    let path = write_config(
        r#"
version = 1

[reactor]
concurrency = 5

[[skills]]
id = "demo"

[skills.source]
repo = "https://example.com/demo.git"

[[skills.targets]]
agent = "custom"
path = "/tmp/agent-skills"
"#,
    );

    let loaded =
        load_from_file(&path, LoadOptions::default()).expect("reactor config should be accepted");
    assert_eq!(loaded.config.reactor.concurrency, 5);
}

#[test]
fn reactor_concurrency_zero_returns_invalid_concurrency_code() {
    let path = write_config(
        r#"
version = 1

[reactor]
concurrency = 0

[[skills]]
id = "demo"

[skills.source]
repo = "https://example.com/demo.git"

[[skills.targets]]
agent = "custom"
path = "/tmp/agent-skills"
"#,
    );

    let err = load_from_file(&path, LoadOptions::default()).expect_err("expected validation error");
    assert!(
        err.to_string().contains("INVALID_CONCURRENCY"),
        "expected INVALID_CONCURRENCY code, got {err}"
    );
}

#[test]
fn reactor_concurrency_above_range_returns_invalid_concurrency_code() {
    let path = write_config(
        r#"
version = 1

[reactor]
concurrency = 101

[[skills]]
id = "demo"

[skills.source]
repo = "https://example.com/demo.git"

[[skills.targets]]
agent = "custom"
path = "/tmp/agent-skills"
"#,
    );

    let err = load_from_file(&path, LoadOptions::default()).expect_err("expected validation error");
    assert!(
        err.to_string().contains("INVALID_CONCURRENCY"),
        "expected INVALID_CONCURRENCY code, got {err}"
    );
}

#[test]
fn phase1_config_remains_valid_in_phase2_loader() {
    let path = write_config(
        r#"
version = 1

[[skills]]
id = "phase1-skill"

[skills.source]
repo = "https://example.com/phase1.git"

[[skills.targets]]
agent = "claude-code"
"#,
    );

    let loaded = load_from_file(&path, LoadOptions::default()).expect("phase1 config should load");
    assert_eq!(loaded.config.skills.len(), 1);
    assert_eq!(loaded.config.skills[0].id, "phase1-skill");
    assert_eq!(loaded.config.reactor.concurrency, 10);
}
