use std::fs;
use std::path::Path;
use std::process::Command;

use eden_skills_core::config::{
    AgentKind, Config, InstallConfig, InstallMode, SafetyConfig, SkillConfig, SourceConfig,
    TargetConfig, VerifyConfig,
};
use eden_skills_core::safety::{analyze_skills, persist_reports, LicenseStatus};
use tempfile::tempdir;

const SKILL_ID: &str = "demo-skill";

#[test]
fn analyze_and_persist_reports_detects_permissive_license_and_risk_labels() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path(), true);

    let storage_root = temp.path().join("storage");
    let repo_path = storage_root.join(SKILL_ID);
    fs::create_dir_all(&storage_root).expect("create storage");
    run_git(
        temp.path(),
        &[
            "clone",
            origin_repo.to_str().expect("origin path utf8"),
            repo_path.to_str().expect("repo path utf8"),
        ],
    );

    let config = test_config(&storage_root);
    let reports = analyze_skills(&config, temp.path()).expect("analyze safety");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].license_status, LicenseStatus::Permissive);
    assert_eq!(reports[0].license_hint.as_deref(), Some("MIT"));
    assert!(
        reports[0]
            .risk_labels
            .iter()
            .any(|label| label == "contains-shell-script"),
        "expected shell script risk label, labels={:?}",
        reports[0].risk_labels
    );

    persist_reports(&reports).expect("persist reports");
    let metadata = fs::read_to_string(repo_path.join(".eden-safety.toml")).expect("read metadata");
    assert!(metadata.contains("license_status = \"permissive\""));
    assert!(metadata.contains("contains-shell-script"));
}

#[test]
fn analyze_reports_unknown_license_when_no_license_file() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path(), false);

    let storage_root = temp.path().join("storage");
    let repo_path = storage_root.join(SKILL_ID);
    fs::create_dir_all(&storage_root).expect("create storage");
    run_git(
        temp.path(),
        &[
            "clone",
            origin_repo.to_str().expect("origin path utf8"),
            repo_path.to_str().expect("repo path utf8"),
        ],
    );

    let config = test_config(&storage_root);
    let reports = analyze_skills(&config, temp.path()).expect("analyze safety");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].license_status, LicenseStatus::Unknown);
}

fn init_origin_repo(base: &Path, with_mit_license: bool) -> std::path::PathBuf {
    let repo = base.join("origin-repo");
    fs::create_dir_all(repo.join("packages").join("browser")).expect("create repo tree");
    fs::write(
        repo.join("packages").join("browser").join("run.sh"),
        "#!/bin/sh\necho hi\n",
    )
    .expect("write script");
    if with_mit_license {
        fs::write(
            repo.join("LICENSE"),
            "MIT License\n\nPermission is hereby granted, free of charge, to any person obtaining a copy...",
        )
        .expect("write license");
    }

    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "test@example.com"]);
    run_git(&repo, &["config", "user.name", "eden-skills-test"]);
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "init"]);
    run_git(&repo, &["branch", "-M", "main"]);
    repo
}

fn test_config(storage_root: &Path) -> Config {
    Config {
        version: 1,
        storage_root: storage_root.display().to_string(),
        skills: vec![SkillConfig {
            id: SKILL_ID.to_string(),
            source: SourceConfig {
                repo: "file:///tmp/origin.git".to_string(),
                subpath: "packages/browser".to_string(),
                r#ref: "main".to_string(),
            },
            install: InstallConfig {
                mode: InstallMode::Symlink,
            },
            targets: vec![TargetConfig {
                agent: AgentKind::Custom,
                expected_path: None,
                path: Some(storage_root.join("targets").display().to_string()),
            }],
            verify: VerifyConfig {
                enabled: false,
                checks: vec![],
            },
            safety: SafetyConfig {
                no_exec_metadata_only: false,
            },
        }],
    }
}

fn run_git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn git");
    if output.status.success() {
        return;
    }

    panic!(
        "git {:?} failed in {}: status={} stderr=`{}` stdout=`{}`",
        args,
        cwd.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr).trim(),
        String::from_utf8_lossy(&output.stdout).trim()
    );
}
