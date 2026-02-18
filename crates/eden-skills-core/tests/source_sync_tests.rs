use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use eden_skills_core::config::{
    AgentKind, Config, InstallConfig, InstallMode, SafetyConfig, SkillConfig, SourceConfig,
    TargetConfig, VerifyConfig,
};
use eden_skills_core::source::{sync_sources, SyncFailureStage};
use tempfile::tempdir;

const SKILL_ID: &str = "demo-skill";

#[test]
fn sync_sources_tracks_cloned_skipped_and_updated_counts() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    fs::create_dir_all(&storage_root).expect("create storage");
    let config = test_config(&storage_root, &as_file_url(&origin_repo), "main");

    let first = sync_sources(&config, temp.path()).expect("first sync");
    assert_eq!(first.cloned, 1);
    assert_eq!(first.updated, 0);
    assert_eq!(first.skipped, 0);
    assert_eq!(first.failed, 0);

    let second = sync_sources(&config, temp.path()).expect("second sync");
    assert_eq!(second.cloned, 0);
    assert_eq!(second.updated, 0);
    assert_eq!(second.skipped, 1);
    assert_eq!(second.failed, 0);

    fs::write(origin_repo.join("packages/browser/README.txt"), "v2\n")
        .expect("write origin update");
    run_git(&origin_repo, &["add", "."]);
    run_git(&origin_repo, &["commit", "-m", "update"]);

    let third = sync_sources(&config, temp.path()).expect("third sync");
    assert_eq!(third.cloned, 0);
    assert_eq!(third.updated, 1);
    assert_eq!(third.skipped, 0);
    assert_eq!(third.failed, 0);
}

#[test]
fn sync_sources_reports_clone_failure_stage() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    fs::create_dir_all(&storage_root).expect("create storage");

    let missing_repo = temp.path().join("missing-origin");
    let config = test_config(&storage_root, &as_file_url(&missing_repo), "main");
    let summary = sync_sources(&config, temp.path()).expect("sync summary");

    assert_eq!(summary.failed, 1);
    assert_eq!(summary.failures.len(), 1);
    assert_eq!(summary.failures[0].skill_id, SKILL_ID);
    assert_eq!(summary.failures[0].stage, SyncFailureStage::Clone);
    assert!(
        !summary.failures[0].detail.is_empty(),
        "expected diagnostic detail for clone failure"
    );
}

#[test]
fn sync_sources_reports_fetch_failure_stage() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    let repo_dir = storage_root.join(SKILL_ID);
    fs::create_dir_all(&repo_dir).expect("create fake repo dir");
    fs::write(repo_dir.join(".git"), "not-a-repo").expect("write fake .git marker");

    let config = test_config(&storage_root, &as_file_url(&origin_repo), "main");
    let summary = sync_sources(&config, temp.path()).expect("sync summary");

    assert_eq!(summary.failed, 1);
    assert_eq!(summary.failures.len(), 1);
    assert_eq!(summary.failures[0].skill_id, SKILL_ID);
    assert_eq!(summary.failures[0].stage, SyncFailureStage::Fetch);
}

#[test]
fn sync_sources_reports_checkout_failure_stage() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    fs::create_dir_all(&storage_root).expect("create storage");

    let config = test_config(
        &storage_root,
        &as_file_url(&origin_repo),
        "definitely-missing-ref",
    );
    let summary = sync_sources(&config, temp.path()).expect("sync summary");

    assert_eq!(summary.failed, 1);
    assert_eq!(summary.failures.len(), 1);
    assert_eq!(summary.failures[0].skill_id, SKILL_ID);
    assert_eq!(summary.failures[0].stage, SyncFailureStage::Checkout);
}

#[test]
fn sync_sources_continues_after_failure_and_aggregates_results() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    fs::create_dir_all(&storage_root).expect("create storage");

    let missing_repo = temp.path().join("missing-origin");
    let config = Config {
        version: 1,
        storage_root: storage_root.display().to_string(),
        skills: vec![
            SkillConfig {
                id: "good-skill".to_string(),
                source: SourceConfig {
                    repo: as_file_url(&origin_repo),
                    subpath: ".".to_string(),
                    r#ref: "main".to_string(),
                },
                install: InstallConfig {
                    mode: InstallMode::Symlink,
                },
                targets: vec![TargetConfig {
                    agent: AgentKind::Custom,
                    expected_path: None,
                    path: Some(storage_root.join("targets").display().to_string()),
                    environment: "local".to_string(),
                }],
                verify: VerifyConfig {
                    enabled: false,
                    checks: vec![],
                },
                safety: SafetyConfig {
                    no_exec_metadata_only: false,
                },
            },
            SkillConfig {
                id: "bad-skill".to_string(),
                source: SourceConfig {
                    repo: as_file_url(&missing_repo),
                    subpath: ".".to_string(),
                    r#ref: "main".to_string(),
                },
                install: InstallConfig {
                    mode: InstallMode::Symlink,
                },
                targets: vec![TargetConfig {
                    agent: AgentKind::Custom,
                    expected_path: None,
                    path: Some(storage_root.join("targets").display().to_string()),
                    environment: "local".to_string(),
                }],
                verify: VerifyConfig {
                    enabled: false,
                    checks: vec![],
                },
                safety: SafetyConfig {
                    no_exec_metadata_only: false,
                },
            },
        ],
    };

    let summary = sync_sources(&config, temp.path()).expect("sync summary");
    assert_eq!(summary.cloned, 1);
    assert_eq!(summary.updated, 0);
    assert_eq!(summary.skipped, 0);
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.failures.len(), 1);
    assert_eq!(summary.failures[0].skill_id, "bad-skill");
    assert_eq!(summary.failures[0].stage, SyncFailureStage::Clone);
    assert!(
        storage_root.join("good-skill").join(".git").exists(),
        "successful skill should still be cloned when another skill fails"
    );
}

fn init_origin_repo(base: &Path) -> PathBuf {
    let repo = base.join("origin-repo");
    fs::create_dir_all(repo.join("packages/browser")).expect("create repo tree");
    fs::write(repo.join("packages/browser/README.txt"), "v1\n").expect("write seed file");

    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "test@example.com"]);
    run_git(&repo, &["config", "user.name", "eden-skills-test"]);
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "init"]);
    run_git(&repo, &["branch", "-M", "main"]);
    repo
}

fn as_file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

fn test_config(storage_root: &Path, repo_url: &str, reference: &str) -> Config {
    Config {
        version: 1,
        storage_root: storage_root.display().to_string(),
        skills: vec![SkillConfig {
            id: SKILL_ID.to_string(),
            source: SourceConfig {
                repo: repo_url.to_string(),
                subpath: ".".to_string(),
                r#ref: reference.to_string(),
            },
            install: InstallConfig {
                mode: InstallMode::Symlink,
            },
            targets: vec![TargetConfig {
                agent: AgentKind::Custom,
                expected_path: None,
                path: Some(storage_root.join("targets").display().to_string()),
                environment: "local".to_string(),
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
