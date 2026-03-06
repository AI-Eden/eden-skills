mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use eden_skills_core::source::repo_cache_key;
use tempfile::tempdir;

#[test]
fn remote_install_reuses_discovery_clone_when_repo_cache_is_empty() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_remote_skill_repo(temp.path(), "remote-skill-repo", "remote-skill");
    let config_path = temp.path().join("skills.toml");
    let git_log = temp.path().join("git-clones.log");
    let repo_url = as_file_url(&repo_dir);

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("EDEN_SKILLS_TEST_GIT_CLONE_LOG", &git_log)
        .args(["install", &repo_url, "--config"])
        .arg(&config_path)
        .output()
        .expect("run remote install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "remote install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let storage_root = home_dir.join(".eden-skills").join("skills");
    let cache_dir = storage_root
        .join(".repos")
        .join(repo_cache_key(&repo_url, "main"));

    assert!(
        cache_dir.join(".git").exists(),
        "expected repo cache checkout"
    );
    assert_eq!(
        clone_count(&git_log),
        1,
        "expected discovery clone reuse, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn remote_install_falls_back_to_fresh_clone_when_cache_move_fails() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_remote_skill_repo(temp.path(), "rename-fallback-repo", "fallback-skill");
    let config_path = temp.path().join("skills.toml");
    let git_log = temp.path().join("git-clones.log");
    let repo_url = as_file_url(&repo_dir);

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("EDEN_SKILLS_TEST_GIT_CLONE_LOG", &git_log)
        .env("EDEN_SKILLS_TEST_FORCE_DISCOVERY_RENAME_FAIL", "1")
        .args(["install", &repo_url, "--config"])
        .arg(&config_path)
        .output()
        .expect("run remote install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "remote install should succeed when rename fallback is forced, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let storage_root = home_dir.join(".eden-skills").join("skills");
    let cache_dir = storage_root
        .join(".repos")
        .join(repo_cache_key(&repo_url, "main"));

    assert!(
        cache_dir.join(".git").exists(),
        "expected repo cache checkout"
    );
    assert_eq!(
        clone_count(&git_log),
        2,
        "forced rename failure should fall back to a fresh cache clone, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn local_path_install_keeps_using_per_skill_storage_without_repo_cache() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let source_dir = temp.path().join("local-skill");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(
        source_dir.join("SKILL.md"),
        r#"---
name: local-skill
description: Local skill
---
"#,
    )
    .expect("write skill");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./local-skill", "--config"])
        .arg(&config_path)
        .output()
        .expect("run local install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "local install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let storage_root = home_dir.join(".eden-skills").join("skills");
    assert!(
        storage_root.join("local-skill").exists(),
        "expected local source staging under per-skill storage"
    );
    assert!(
        !storage_root.join(".repos").exists(),
        "local installs must not create repo cache directories"
    );
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

fn init_remote_skill_repo(base: &Path, repo_name: &str, skill_name: &str) -> PathBuf {
    let repo = base.join(repo_name);
    fs::create_dir_all(&repo).expect("create repo dir");
    fs::write(
        repo.join("SKILL.md"),
        format!(
            r#"---
name: {skill_name}
description: Remote skill
---
"#
        ),
    )
    .expect("write skill");
    fs::write(repo.join("README.md"), "seed\n").expect("write readme");

    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "test@example.com"]);
    run_git(&repo, &["config", "user.name", "eden-skills-test"]);
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "init"]);
    run_git(&repo, &["branch", "-M", "main"]);
    repo
}

fn clone_count(log_path: &Path) -> usize {
    fs::read_to_string(log_path)
        .unwrap_or_default()
        .lines()
        .filter(|line| *line == "clone")
        .count()
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

fn as_file_url(path: &Path) -> String {
    let mut normalized = path.display().to_string().replace('\\', "/");
    if normalized
        .as_bytes()
        .get(1)
        .is_some_and(|candidate| *candidate == b':')
    {
        normalized.insert(0, '/');
    }
    format!("file://{normalized}")
}
