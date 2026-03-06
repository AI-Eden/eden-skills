use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use eden_skills_core::config::{
    AgentKind, Config, InstallConfig, InstallMode, ReactorConfig, SafetyConfig, SkillConfig,
    SourceConfig, TargetConfig, VerifyConfig,
};
use eden_skills_core::plan::{build_plan, Action};
use eden_skills_core::source::{
    normalize_repo_url, repo_cache_key, resolve_skill_source_path, sanitize_ref, sync_sources,
};
use tempfile::tempdir;

#[test]
fn normalize_repo_url_matches_phase_295_spec_examples() {
    assert_eq!(
        normalize_repo_url("https://github.com/vercel-labs/agent-skills.git"),
        "github.com_vercel-labs_agent-skills"
    );
    assert_eq!(
        normalize_repo_url("git@github.com:user/repo.git"),
        "github.com_user_repo"
    );
    assert_eq!(
        normalize_repo_url("https://github.com/AI-Eden/eden-skills"),
        "github.com_ai-eden_eden-skills"
    );
}

#[test]
fn sanitize_ref_matches_phase_295_spec_examples() {
    assert_eq!(sanitize_ref("main"), "main");
    assert_eq!(sanitize_ref("v2.0"), "v2.0");
    assert_eq!(sanitize_ref("refs/heads/main"), "refs_heads_main");
}

#[test]
fn resolve_skill_source_path_uses_repo_cache_for_remote_sources() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let skill = test_skill(
        "browser-tool",
        "https://github.com/AI-Eden/eden-skills",
        "packages/browser",
        "main",
    );

    let expected = storage_root
        .join(".repos")
        .join("github.com_ai-eden_eden-skills@main")
        .join("packages/browser");

    assert_eq!(resolve_skill_source_path(&storage_root, &skill), expected);
}

#[test]
fn resolve_skill_source_path_keeps_local_sources_under_per_skill_storage() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let local_repo = temp.path().join("local-source");
    let skill = test_skill(
        "browser-tool",
        &local_repo.display().to_string(),
        "packages/browser",
        "main",
    );

    let expected = storage_root.join("browser-tool").join("packages/browser");

    assert_eq!(resolve_skill_source_path(&storage_root, &skill), expected);
}

#[test]
fn build_plan_uses_repo_cache_source_path_for_remote_skills() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    let config = Config {
        version: 1,
        storage_root: storage_root.display().to_string(),
        reactor: ReactorConfig::default(),
        skills: vec![test_skill_config(
            "browser-tool",
            "https://github.com/AI-Eden/eden-skills",
            "packages/browser",
            "main",
            &target_root,
        )],
    };
    let expected_source = resolve_skill_source_path(&storage_root, &config.skills[0]);
    fs::create_dir_all(&expected_source).expect("create cached source directory");
    fs::write(expected_source.join("README.md"), "cached repo content\n")
        .expect("write cached source file");

    let plan = build_plan(&config, temp.path()).expect("build plan");

    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].action, Action::Create);
    assert_eq!(plan[0].source_path, expected_source.display().to_string());
}

#[test]
fn sync_sources_creates_repo_cache_directory_on_first_sync() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    fs::create_dir_all(&storage_root).expect("create storage");
    let repo_url = as_file_url(&origin_repo);
    let config = test_config(
        &storage_root,
        vec![test_skill_config(
            "browser-tool",
            &repo_url,
            "packages/browser",
            "main",
            temp.path(),
        )],
    );

    let summary = sync_sources(&config, temp.path()).expect("sync sources");
    let cache_dir = storage_root
        .join(".repos")
        .join(repo_cache_key(&repo_url, "main"));

    assert_eq!(summary.cloned, 1);
    assert_eq!(summary.failed, 0);
    assert!(
        storage_root.join(".repos").is_dir(),
        "expected .repos directory"
    );
    assert!(
        cache_dir.join(".git").exists(),
        "expected cloned cache repo"
    );
}

#[test]
fn sync_sources_groups_same_repo_and_ref_into_one_clone() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    fs::create_dir_all(&storage_root).expect("create storage");
    let repo_url = as_file_url(&origin_repo);
    let config = test_config(
        &storage_root,
        vec![
            test_skill_config(
                "browser-tool",
                &repo_url,
                "packages/browser",
                "main",
                temp.path(),
            ),
            test_skill_config(
                "browser-tool-docs",
                &repo_url,
                "packages/browser",
                "main",
                temp.path(),
            ),
        ],
    );

    let summary = sync_sources(&config, temp.path()).expect("sync sources");
    let cache_entries = read_cache_dir_names(&storage_root);

    assert_eq!(summary.cloned, 1);
    assert_eq!(summary.updated, 0);
    assert_eq!(summary.skipped, 0);
    assert_eq!(summary.failed, 0);
    assert_eq!(cache_entries.len(), 1, "expected one repo cache directory");
    assert!(
        !storage_root.join("browser-tool").exists(),
        "legacy per-skill checkout should not be created for remote sync"
    );
    assert!(
        !storage_root.join("browser-tool-docs").exists(),
        "legacy per-skill checkout should not be created for remote sync"
    );
}

#[test]
fn sync_sources_separates_cache_directories_by_ref() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    run_git(&origin_repo, &["tag", "v2.0"]);
    let storage_root = temp.path().join("storage");
    fs::create_dir_all(&storage_root).expect("create storage");
    let repo_url = as_file_url(&origin_repo);
    let config = test_config(
        &storage_root,
        vec![
            test_skill_config(
                "browser-tool-main",
                &repo_url,
                "packages/browser",
                "main",
                temp.path(),
            ),
            test_skill_config(
                "browser-tool-tag",
                &repo_url,
                "packages/browser",
                "v2.0",
                temp.path(),
            ),
        ],
    );

    let summary = sync_sources(&config, temp.path()).expect("sync sources");
    let cache_entries = read_cache_dir_names(&storage_root);

    assert_eq!(summary.cloned, 2);
    assert_eq!(summary.failed, 0);
    assert_eq!(
        cache_entries.len(),
        2,
        "expected separate cache directories per ref"
    );
    assert!(cache_entries.contains(&repo_cache_key(&repo_url, "main")));
    assert!(cache_entries.contains(&repo_cache_key(&repo_url, "v2.0")));
}

#[test]
fn legacy_per_skill_directories_do_not_block_repo_cache_sync() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    let legacy_dir = storage_root.join("browser-tool");
    fs::create_dir_all(&legacy_dir).expect("create legacy per-skill dir");
    fs::write(legacy_dir.join("README.md"), "legacy checkout\n").expect("write legacy file");
    let repo_url = as_file_url(&origin_repo);
    let config = test_config(
        &storage_root,
        vec![test_skill_config(
            "browser-tool",
            &repo_url,
            "packages/browser",
            "main",
            temp.path(),
        )],
    );

    let summary = sync_sources(&config, temp.path()).expect("sync sources");
    let cache_dir = storage_root
        .join(".repos")
        .join(repo_cache_key(&repo_url, "main"));

    assert_eq!(summary.cloned, 1);
    assert_eq!(summary.failed, 0);
    assert!(cache_dir.join(".git").exists(), "expected cache clone");
    assert_eq!(
        fs::read_to_string(legacy_dir.join("README.md")).expect("read legacy file"),
        "legacy checkout\n"
    );
}

fn init_origin_repo(base: &Path) -> PathBuf {
    let repo = base.join("origin-repo");
    fs::create_dir_all(repo.join("packages").join("browser")).expect("create repo tree");
    fs::write(
        repo.join("packages").join("browser").join("README.txt"),
        "v1\n",
    )
    .expect("write seed file");

    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "test@example.com"]);
    run_git(&repo, &["config", "user.name", "eden-skills-test"]);
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "init"]);
    run_git(&repo, &["branch", "-M", "main"]);
    repo
}

fn read_cache_dir_names(storage_root: &Path) -> Vec<String> {
    let mut entries = fs::read_dir(storage_root.join(".repos"))
        .expect("read repo cache directory")
        .map(|entry| {
            entry
                .expect("cache entry")
                .file_name()
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn as_file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

fn test_config(storage_root: &Path, skills: Vec<SkillConfig>) -> Config {
    Config {
        version: 1,
        storage_root: storage_root.display().to_string(),
        reactor: ReactorConfig::default(),
        skills,
    }
}

fn test_skill(skill_id: &str, repo_url: &str, subpath: &str, reference: &str) -> SkillConfig {
    test_skill_config(skill_id, repo_url, subpath, reference, Path::new("."))
}

fn test_skill_config(
    skill_id: &str,
    repo_url: &str,
    subpath: &str,
    reference: &str,
    target_root: &Path,
) -> SkillConfig {
    SkillConfig {
        id: skill_id.to_string(),
        source: SourceConfig {
            repo: repo_url.to_string(),
            subpath: subpath.to_string(),
            r#ref: reference.to_string(),
        },
        install: InstallConfig {
            mode: InstallMode::Symlink,
        },
        targets: vec![TargetConfig {
            agent: AgentKind::Custom,
            expected_path: None,
            path: Some(target_root.join("targets").display().to_string()),
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

fn run_git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn git");
    assert!(
        output.status.success(),
        "git {:?} failed in {}: status={} stderr=`{}` stdout=`{}`",
        args,
        cwd.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr).trim(),
        String::from_utf8_lossy(&output.stdout).trim()
    );
}
