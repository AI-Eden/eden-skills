mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use eden_skills_cli::commands::{apply, repair, CommandOptions};
use eden_skills_core::config::InstallMode;
use eden_skills_core::error::EdenError;
use serde_json::Value;
use tempfile::tempdir;

use common::{
    as_file_url, default_options, expected_source_path, expected_target_path, init_origin_repo,
    run_git_cmd, write_config, write_config_with_safety, SKILL_ID,
};

#[test]
fn apply_no_exec_metadata_only_skips_target_mutation_and_writes_metadata() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let script_path = origin_repo.join("packages").join("browser").join("run.sh");
    fs::write(&script_path, "#!/bin/sh\necho hi\n").expect("write script");
    run_git_cmd(&origin_repo, &["add", "."]);
    run_git_cmd(&origin_repo, &["commit", "-m", "add script"]);

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config_with_safety(
        temp.path(),
        &as_file_url(&origin_repo),
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
        true,
    );

    apply(
        config_path.to_str().expect("config path"),
        default_options(),
    )
    .expect("apply with no_exec_metadata_only");

    let target_path = expected_target_path(&target_root);
    assert!(
        !target_path.exists(),
        "target should not be created when no_exec_metadata_only=true"
    );

    let source_path = expected_source_path(&storage_root);
    assert!(source_path.exists(), "source should still be synchronized");

    let metadata_path = storage_root.join(SKILL_ID).join(".eden-safety.toml");
    let metadata = fs::read_to_string(&metadata_path).expect("read safety metadata");
    assert!(metadata.contains("version = 1"));
    assert!(metadata.contains("no_exec_metadata_only = true"));
    assert!(metadata.contains("license_status = \"unknown\""));
    assert!(metadata.contains("contains-shell-script"));
}

#[test]
fn doctor_reports_safety_findings() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let script_path = origin_repo.join("packages").join("browser").join("run.sh");
    fs::write(&script_path, "#!/bin/sh\necho hi\n").expect("write script");
    run_git_cmd(&origin_repo, &["add", "."]);
    run_git_cmd(&origin_repo, &["commit", "-m", "add script"]);

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config_with_safety(
        temp.path(),
        &as_file_url(&origin_repo),
        InstallMode::Symlink.as_str(),
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
        true,
    );

    apply(
        config_path.to_str().expect("config path"),
        CommandOptions {
            strict: false,
            json: false,
        },
    )
    .expect("apply before doctor");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["doctor", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor --json");

    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor should succeed without strict mode, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("doctor should output json");
    let findings = payload["findings"]
        .as_array()
        .expect("findings should be an array");

    let has_license_unknown = findings
        .iter()
        .any(|f| f["code"] == "LICENSE_UNKNOWN" && f["severity"] == "warning");
    let has_risk_review = findings
        .iter()
        .any(|f| f["code"] == "RISK_REVIEW_REQUIRED" && f["severity"] == "warning");
    let has_no_exec = findings
        .iter()
        .any(|f| f["code"] == "NO_EXEC_METADATA_ONLY" && f["severity"] == "warning");

    assert!(has_license_unknown, "expected LICENSE_UNKNOWN in findings");
    assert!(has_risk_review, "expected RISK_REVIEW_REQUIRED in findings");
    assert!(has_no_exec, "expected NO_EXEC_METADATA_ONLY in findings");
}

#[test]
fn apply_sync_failure_still_writes_safety_metadata() {
    let temp = tempdir().expect("tempdir");
    let missing_repo = temp.path().join("missing-origin-repo");

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config(
        temp.path(),
        &as_file_url(&missing_repo),
        InstallMode::Symlink.as_str(),
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["apply", "--config"])
        .arg(&config_path)
        .output()
        .expect("run apply");

    assert_eq!(
        output.status.code(),
        Some(1),
        "apply should fail with source sync error, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata_path = storage_root.join(SKILL_ID).join(".eden-safety.toml");
    let metadata = fs::read_to_string(&metadata_path).expect("read safety metadata");
    assert!(metadata.contains("version = 1"));
    assert!(metadata.contains("license_status = \"unknown\""));
}

#[test]
fn apply_mixed_skills_skips_verify_only_for_no_exec_skill() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_multiskill_safety_config(
        temp.path(),
        &as_file_url(&origin_repo),
        &storage_root,
        &target_root,
        &[
            SkillEntry {
                id: "regular-skill",
                verify_checks: &["path-exists", "target-resolves", "is-symlink"],
                no_exec_metadata_only: false,
            },
            SkillEntry {
                id: "metadata-only-skill",
                verify_checks: &["content-present"],
                no_exec_metadata_only: true,
            },
        ],
    );

    apply(
        config_path.to_str().expect("config path"),
        CommandOptions {
            strict: false,
            json: false,
        },
    )
    .expect("apply mixed skills");

    let regular_target = target_root.join("regular-skill");
    let metadata_only_target = target_root.join("metadata-only-skill");
    assert!(regular_target.exists(), "regular skill target should exist");
    assert!(
        !metadata_only_target.exists(),
        "metadata-only skill target should not be mutated"
    );
}

#[test]
fn apply_mixed_skills_still_verifies_non_no_exec_skill() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_multiskill_safety_config(
        temp.path(),
        &as_file_url(&origin_repo),
        &storage_root,
        &target_root,
        &[
            SkillEntry {
                id: "regular-skill",
                verify_checks: &["content-present"],
                no_exec_metadata_only: false,
            },
            SkillEntry {
                id: "metadata-only-skill",
                verify_checks: &["content-present"],
                no_exec_metadata_only: true,
            },
        ],
    );

    let err = apply(
        config_path.to_str().expect("config path"),
        CommandOptions {
            strict: false,
            json: false,
        },
    )
    .expect_err("non-no-exec skill verification should still run");
    assert!(
        matches!(err, EdenError::Runtime(_)),
        "expected runtime verification failure, got: {err}"
    );
    assert!(
        err.to_string().contains("post-apply verification failed"),
        "expected post-apply verification message, got: {err}"
    );
}

#[test]
fn apply_strict_ignores_conflicts_for_no_exec_skill() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_multiskill_safety_config(
        temp.path(),
        &as_file_url(&origin_repo),
        &storage_root,
        &target_root,
        &[
            SkillEntry {
                id: "regular-skill",
                verify_checks: &["path-exists", "target-resolves", "is-symlink"],
                no_exec_metadata_only: false,
            },
            SkillEntry {
                id: "metadata-only-skill",
                verify_checks: &["path-exists", "target-resolves", "is-symlink"],
                no_exec_metadata_only: true,
            },
        ],
    );

    let conflicted_target = target_root.join("metadata-only-skill");
    fs::create_dir_all(&conflicted_target).expect("create conflicting target");
    fs::write(conflicted_target.join("manual.txt"), "manual content").expect("write manual file");

    apply(
        config_path.to_str().expect("config path"),
        CommandOptions {
            strict: true,
            json: false,
        },
    )
    .expect("strict apply should ignore no-exec conflicts");
}

#[test]
fn repair_strict_ignores_conflicts_for_no_exec_skill() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_multiskill_safety_config(
        temp.path(),
        &as_file_url(&origin_repo),
        &storage_root,
        &target_root,
        &[
            SkillEntry {
                id: "regular-skill",
                verify_checks: &["path-exists", "target-resolves", "is-symlink"],
                no_exec_metadata_only: false,
            },
            SkillEntry {
                id: "metadata-only-skill",
                verify_checks: &["path-exists", "target-resolves", "is-symlink"],
                no_exec_metadata_only: true,
            },
        ],
    );

    let conflicted_target = target_root.join("metadata-only-skill");
    fs::create_dir_all(&conflicted_target).expect("create conflicting target");
    fs::write(conflicted_target.join("manual.txt"), "manual content").expect("write manual file");

    repair(
        config_path.to_str().expect("config path"),
        CommandOptions {
            strict: true,
            json: false,
        },
    )
    .expect("strict repair should ignore no-exec conflicts");
}

#[derive(Clone, Copy)]
struct SkillEntry<'a> {
    id: &'a str,
    verify_checks: &'a [&'a str],
    no_exec_metadata_only: bool,
}

fn write_multiskill_safety_config(
    base: &Path,
    repo_url: &str,
    storage_root: &Path,
    target_root: &Path,
    skills: &[SkillEntry<'_>],
) -> PathBuf {
    let mut config = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n\n",
        toml_escape_path(storage_root)
    );

    for skill in skills {
        config.push_str("[[skills]]\n");
        config.push_str(&format!("id = \"{}\"\n\n", toml_escape_string(skill.id)));

        config.push_str("[skills.source]\n");
        config.push_str(&format!("repo = \"{}\"\n", toml_escape_string(repo_url)));
        config.push_str("subpath = \"packages/browser\"\n");
        config.push_str("ref = \"main\"\n\n");

        config.push_str("[skills.install]\n");
        config.push_str("mode = \"symlink\"\n\n");

        config.push_str("[[skills.targets]]\n");
        config.push_str("agent = \"custom\"\n");
        config.push_str(&format!("path = \"{}\"\n\n", toml_escape_path(target_root)));

        let checks = skill
            .verify_checks
            .iter()
            .map(|check| format!("\"{}\"", toml_escape_string(check)))
            .collect::<Vec<_>>()
            .join(", ");
        config.push_str("[skills.verify]\n");
        config.push_str("enabled = true\n");
        config.push_str(&format!("checks = [{}]\n\n", checks));

        config.push_str("[skills.safety]\n");
        config.push_str(&format!(
            "no_exec_metadata_only = {}\n\n",
            skill.no_exec_metadata_only
        ));
    }

    let config_path = base.join("skills.toml");
    fs::write(&config_path, config).expect("write config");
    config_path
}

fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

fn toml_escape_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
