mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{assert_success, eden_command, path_to_file_url, run_git_cmd};
use eden_skills_core::source::resolve_repo_cache_root;
use serde_json::Value;
use tempfile::{tempdir, TempDir};

struct InstallFixture {
    temp: TempDir,
    home_dir: PathBuf,
    config_path: PathBuf,
    source_root: PathBuf,
}

#[test]
fn tm_p297_001_update_deduplicates_fetches_for_shared_repo_cache_keys() {
    let fixture = setup_remote_install_fixture(&["alpha-skill", "beta-skill"]);
    let fetch_log = fixture.temp.path().join("git-fetches.log");

    let update_output = run_command(
        &fixture,
        &[("EDEN_SKILLS_TEST_GIT_FETCH_LOG", fetch_log.as_os_str())],
        &["update"],
    );
    assert_success(&update_output);

    assert_eq!(
        fetch_count(&fetch_log),
        1,
        "update should fetch a shared repo-cache checkout once even when multiple skills share it, stdout={} stderr={}",
        String::from_utf8_lossy(&update_output.stdout),
        String::from_utf8_lossy(&update_output.stderr)
    );
}

#[test]
fn tm_p297_002_update_broadcasts_refresh_status_to_all_skills_in_a_shared_repo_group() {
    let fixture = setup_remote_install_fixture(&["alpha-skill", "beta-skill"]);
    let fetch_log = fixture.temp.path().join("git-fetches.log");

    commit_file(
        &fixture.source_root,
        "alpha-skill/README.md",
        "alpha-v2\n",
        "upstream update",
    );

    let update_output = run_command(
        &fixture,
        &[("EDEN_SKILLS_TEST_GIT_FETCH_LOG", fetch_log.as_os_str())],
        &["update"],
    );
    assert_success(&update_output);
    let stdout = String::from_utf8_lossy(&update_output.stdout);

    assert_eq!(
        fetch_count(&fetch_log),
        1,
        "shared repo-cache refresh should still execute a single fetch, stdout={stdout}"
    );
    assert!(
        stdout.contains("alpha-skill"),
        "refresh table should include alpha-skill, stdout={stdout}"
    );
    assert!(
        stdout.contains("beta-skill"),
        "refresh table should include beta-skill, stdout={stdout}"
    );
    assert_eq!(
        stdout.matches("new commit").count(),
        2,
        "both skills sharing the refreshed repo cache should receive the broadcast status, stdout={stdout}"
    );
}

#[test]
fn tm_p297_003_update_clears_stale_index_lock_before_fetching() {
    let fixture = setup_remote_install_fixture(&["lock-skill"]);
    let repo_dir = resolve_repo_cache_root(
        &storage_root(&fixture),
        &path_to_file_url(&fixture.source_root),
        "main",
    );
    let index_lock = repo_dir.join(".git").join("index.lock");
    let warning_fragment = format!("removed stale git lock `{}`", index_lock.display());

    write_stale_lock(&index_lock);

    let first_update = run_command(&fixture, &[], &["update"]);
    assert_success(&first_update);
    let first_stdout = String::from_utf8_lossy(&first_update.stdout);
    let first_stderr = String::from_utf8_lossy(&first_update.stderr);

    assert!(
        !first_stdout.contains("failed"),
        "stale index.lock should not leave the refresh in a failed state, stdout={first_stdout} stderr={first_stderr}"
    );
    assert!(
        !index_lock.exists(),
        "stale index.lock should be removed before fetch"
    );
    assert!(
        first_stderr.contains(&warning_fragment),
        "stale index.lock cleanup should emit a warning, stderr={first_stderr}"
    );

    let second_update = run_command(&fixture, &[], &["update"]);
    assert_success(&second_update);
    assert!(
        !String::from_utf8_lossy(&second_update.stdout).contains("failed"),
        "a consecutive update after stale lock cleanup should also succeed, stdout={} stderr={}",
        String::from_utf8_lossy(&second_update.stdout),
        String::from_utf8_lossy(&second_update.stderr)
    );
}

#[test]
fn tm_p297_004_update_clears_stale_shallow_lock_before_fetching() {
    let fixture = setup_remote_install_fixture(&["lock-skill"]);
    let repo_dir = resolve_repo_cache_root(
        &storage_root(&fixture),
        &path_to_file_url(&fixture.source_root),
        "main",
    );
    let shallow_lock = repo_dir.join(".git").join("shallow.lock");
    let warning_fragment = format!("removed stale git lock `{}`", shallow_lock.display());

    write_stale_lock(&shallow_lock);

    let update_output = run_command(&fixture, &[], &["update"]);
    assert_success(&update_output);
    let stdout = String::from_utf8_lossy(&update_output.stdout);
    let stderr = String::from_utf8_lossy(&update_output.stderr);

    assert!(
        !stdout.contains("failed"),
        "stale shallow.lock should not leave the refresh in a failed state, stdout={stdout} stderr={stderr}"
    );
    assert!(
        !shallow_lock.exists(),
        "stale shallow.lock should be removed before fetch"
    );
    assert!(
        stderr.contains(&warning_fragment),
        "stale shallow.lock cleanup should emit a warning, stderr={stderr}"
    );
}

#[test]
fn tm_p297_005_local_source_skills_are_not_grouped_by_repo_cache_key() {
    let fixture = setup_local_install_fixture(&["alpha-skill", "beta-skill"]);
    let fetch_log = fixture.temp.path().join("git-fetches.log");

    let update_output = run_command(
        &fixture,
        &[("EDEN_SKILLS_TEST_GIT_FETCH_LOG", fetch_log.as_os_str())],
        &["update"],
    );
    assert_success(&update_output);

    assert_eq!(
        fetch_count(&fetch_log),
        2,
        "local-source skills should keep one refresh fetch per staged skill copy, stdout={} stderr={}",
        String::from_utf8_lossy(&update_output.stdout),
        String::from_utf8_lossy(&update_output.stderr)
    );
}

#[test]
fn tm_p297_006_update_json_keeps_per_skill_rows_after_deduplicated_fetch() {
    let fixture = setup_remote_install_fixture(&["alpha-skill", "beta-skill"]);
    let fetch_log = fixture.temp.path().join("git-fetches.log");

    commit_file(
        &fixture.source_root,
        "beta-skill/README.md",
        "beta-v2\n",
        "upstream update",
    );

    let update_output = run_command(
        &fixture,
        &[("EDEN_SKILLS_TEST_GIT_FETCH_LOG", fetch_log.as_os_str())],
        &["update", "--json"],
    );
    assert_success(&update_output);
    let stdout = String::from_utf8_lossy(&update_output.stdout);
    let payload: Value = serde_json::from_str(&stdout).unwrap_or_else(|err| {
        panic!("update --json should emit valid JSON, err={err} stdout={stdout}")
    });

    assert_eq!(
        fetch_count(&fetch_log),
        1,
        "shared repo-cache refresh should still execute a single fetch in JSON mode, stdout={stdout}"
    );

    let skills = payload
        .get("skills")
        .and_then(Value::as_array)
        .unwrap_or_else(|| {
            panic!("expected `skills` array in update JSON payload, json={payload}")
        });
    let mut ids = skills
        .iter()
        .filter_map(|entry| entry.get("id").and_then(Value::as_str))
        .collect::<Vec<_>>();
    ids.sort_unstable();

    assert_eq!(
        ids,
        vec!["alpha-skill", "beta-skill"],
        "deduplicated refresh should still emit one JSON row per skill, json={payload}"
    );
    assert!(
        skills
            .iter()
            .all(|entry| entry.get("status").and_then(Value::as_str) == Some("new-commit")),
        "all grouped skills should receive the same broadcast refresh status, json={payload}"
    );
}

fn setup_remote_install_fixture(skill_names: &[&str]) -> InstallFixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let target_root = temp.path().join("agent-target");
    fs::create_dir_all(&home_dir).expect("create HOME");
    fs::create_dir_all(&target_root).expect("create target root");
    let config_path = temp.path().join("skills.toml");
    let source_root = init_multi_skill_repo(temp.path(), "remote-origin", skill_names);

    let install_output = run_install(
        temp.path(),
        &home_dir,
        &config_path,
        &target_root,
        &path_to_file_url(&source_root),
    );
    assert_success(&install_output);

    InstallFixture {
        temp,
        home_dir,
        config_path,
        source_root,
    }
}

fn setup_local_install_fixture(skill_names: &[&str]) -> InstallFixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let target_root = temp.path().join("agent-target");
    fs::create_dir_all(&home_dir).expect("create HOME");
    fs::create_dir_all(&target_root).expect("create target root");
    let config_path = temp.path().join("skills.toml");
    let source_root = init_multi_skill_repo(temp.path(), "local-source", skill_names);

    run_git_cmd(
        &source_root,
        &[
            "remote",
            "add",
            "origin",
            source_root.to_str().expect("source path utf-8"),
        ],
    );

    let install_output = run_install(
        temp.path(),
        &home_dir,
        &config_path,
        &target_root,
        source_root.to_str().expect("source path utf-8"),
    );
    assert_success(&install_output);

    InstallFixture {
        temp,
        home_dir,
        config_path,
        source_root,
    }
}

fn run_install(
    cwd: &Path,
    home_dir: &Path,
    config_path: &Path,
    target_root: &Path,
    source_arg: &str,
) -> Output {
    let target_arg = format!("custom:{}", target_root.display());
    let mut command = eden_command(home_dir);
    command
        .current_dir(cwd)
        .arg("--color")
        .arg("never")
        .arg("install")
        .arg(source_arg)
        .arg("--all")
        .arg("--target")
        .arg(target_arg)
        .arg("--config")
        .arg(config_path);
    command.output().expect("run install")
}

fn run_command(
    fixture: &InstallFixture,
    envs: &[(&str, &std::ffi::OsStr)],
    command_args: &[&str],
) -> Output {
    let mut command = eden_command(&fixture.home_dir);
    command
        .current_dir(fixture.temp.path())
        .arg("--color")
        .arg("never");
    for (key, value) in envs {
        command.env(key, value);
    }
    command.args(command_args);
    command.arg("--config").arg(&fixture.config_path);
    command.output().expect("run eden-skills command")
}

fn storage_root(fixture: &InstallFixture) -> PathBuf {
    fixture.home_dir.join(".eden-skills").join("skills")
}

fn init_multi_skill_repo(base: &Path, repo_name: &str, skill_names: &[&str]) -> PathBuf {
    let repo = base.join(repo_name);
    fs::create_dir_all(&repo).expect("create repo dir");
    for skill_name in skill_names {
        let skill_dir = repo.join(skill_name);
        fs::create_dir_all(&skill_dir).expect("create skill dir");
        fs::write(
            skill_dir.join("SKILL.md"),
            format!(
                r#"---
name: {skill_name}
description: Test skill
---
"#
            ),
        )
        .expect("write skill");
        fs::write(skill_dir.join("README.md"), format!("{skill_name}\n")).expect("write readme");
    }

    run_git_cmd(&repo, &["init"]);
    run_git_cmd(&repo, &["config", "user.email", common::TEST_GIT_EMAIL]);
    run_git_cmd(&repo, &["config", "user.name", common::TEST_GIT_NAME]);
    run_git_cmd(&repo, &["add", "."]);
    run_git_cmd(&repo, &["commit", "-m", "init"]);
    run_git_cmd(&repo, &["branch", "-M", "main"]);
    repo
}

fn commit_file(repo: &Path, rel_path: &str, content: &str, message: &str) {
    let file_path = repo.join(rel_path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(file_path, content).expect("write commit content");
    run_git_cmd(repo, &["add", "."]);
    run_git_cmd(repo, &["commit", "-m", message]);
}

fn write_stale_lock(lock_path: &Path) {
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent).expect("create lock parent");
    }
    fs::write(lock_path, "stale lock\n").expect("write lock file");
    set_old_mtime(lock_path);
}

#[cfg(unix)]
fn set_old_mtime(path: &Path) {
    let output = Command::new("touch")
        .arg("-t")
        .arg("200001010000")
        .arg(path)
        .output()
        .expect("spawn touch");
    assert!(
        output.status.success(),
        "touch failed for {}: status={} stderr=`{}` stdout=`{}`",
        path.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr).trim(),
        String::from_utf8_lossy(&output.stdout).trim()
    );
}

#[cfg(windows)]
fn set_old_mtime(path: &Path) {
    let escaped = path.display().to_string().replace('\'', "''");
    let script = format!(
        "$(Get-Item -LiteralPath '{escaped}').LastWriteTime = [datetime]'2000-01-01T00:00:00'"
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .expect("spawn powershell");
    assert!(
        output.status.success(),
        "powershell mtime update failed for {}: status={} stderr=`{}` stdout=`{}`",
        path.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr).trim(),
        String::from_utf8_lossy(&output.stdout).trim()
    );
}

fn fetch_count(log_path: &Path) -> usize {
    fs::read_to_string(log_path)
        .unwrap_or_default()
        .lines()
        .filter(|line| *line == "fetch")
        .count()
}

