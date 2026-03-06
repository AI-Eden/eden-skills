mod common;

use std::ffi::OsString;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
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
    let fake_bin_dir = temp.path().join("fake-bin");
    install_git_wrapper(&fake_bin_dir);
    let repo_url = as_file_url(&repo_dir);

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("PATH", prepend_to_path(&fake_bin_dir))
        .env("EDEN_SKILLS_TEST_REAL_GIT", real_git_path())
        .env("EDEN_SKILLS_TEST_GIT_LOG", &git_log)
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
    let fake_bin_dir = temp.path().join("fake-bin");
    install_git_wrapper(&fake_bin_dir);
    let repo_url = as_file_url(&repo_dir);

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("PATH", prepend_to_path(&fake_bin_dir))
        .env("EDEN_SKILLS_TEST_REAL_GIT", real_git_path())
        .env("EDEN_SKILLS_TEST_GIT_LOG", &git_log)
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

fn real_git_path() -> String {
    #[cfg(windows)]
    let output = Command::new("where")
        .arg("git.exe")
        .output()
        .expect("run where git");

    #[cfg(not(windows))]
    let output = Command::new("which")
        .arg("git")
        .output()
        .expect("run which git");

    assert!(
        output.status.success(),
        "failed to resolve git path: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .expect("git path output")
        .trim()
        .to_string()
}

fn prepend_to_path(prefix: &Path) -> OsString {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let mut merged = OsString::new();
    merged.push(prefix.as_os_str());
    merged.push(path_separator());
    merged.push(existing);
    merged
}

#[cfg(windows)]
fn path_separator() -> &'static str {
    ";"
}

#[cfg(not(windows))]
fn path_separator() -> &'static str {
    ":"
}

fn install_git_wrapper(fake_bin_dir: &Path) {
    fs::create_dir_all(fake_bin_dir).expect("create fake bin dir");

    #[cfg(windows)]
    {
        let script_path = fake_bin_dir.join("git.cmd");
        fs::write(
            script_path,
            r#"@echo off
if "%1"=="clone" echo clone>>"%EDEN_SKILLS_TEST_GIT_LOG%"
"%EDEN_SKILLS_TEST_REAL_GIT%" %*
"#,
        )
        .expect("write git wrapper");
    }

    #[cfg(not(windows))]
    {
        let script_path = fake_bin_dir.join("git");
        fs::write(
            &script_path,
            r#"#!/bin/sh
if [ "$1" = "clone" ]; then
  printf 'clone\n' >> "$EDEN_SKILLS_TEST_GIT_LOG"
fi
exec "$EDEN_SKILLS_TEST_REAL_GIT" "$@"
"#,
        )
        .expect("write git wrapper");
        let mut permissions = fs::metadata(&script_path)
            .expect("wrapper metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions).expect("set wrapper permissions");
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
