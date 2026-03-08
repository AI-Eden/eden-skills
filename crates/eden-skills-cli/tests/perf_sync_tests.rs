mod common;

use std::fs;
use std::path::{Path, PathBuf};

use eden_skills_core::source::repo_cache_key;
use tempfile::tempdir;

#[test]
fn remote_install_reuses_discovery_clone_when_repo_cache_is_empty() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_remote_skill_repo(temp.path(), "remote-skill-repo", "remote-skill");
    let config_path = temp.path().join("skills.toml");
    let git_log = temp.path().join("git-clones.log");
    let repo_url = common::path_to_file_url(&repo_dir);

    let output = common::eden_command(&home_dir)
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
    let repo_url = common::path_to_file_url(&repo_dir);

    let output = common::eden_command(&home_dir)
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
    let output = common::eden_command(&home_dir)
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

#[test]
fn tm_p295_031_install_batches_remote_sync_into_one_repo_fetch() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_remote_multi_skill_repo(
        temp.path(),
        "batched-remote-repo",
        &["alpha-skill", "beta-skill"],
    );
    let config_path = temp.path().join("skills.toml");
    let clone_log = temp.path().join("git-clones.log");
    let fetch_log = temp.path().join("git-fetches.log");
    let repo_url = common::path_to_file_url(&repo_dir);

    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env("EDEN_SKILLS_TEST_GIT_CLONE_LOG", &clone_log)
        .env("EDEN_SKILLS_TEST_GIT_FETCH_LOG", &fetch_log)
        .args(["install", &repo_url, "--all", "--config"])
        .arg(&config_path)
        .output()
        .expect("run batched remote install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "batched remote install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        clone_count(&clone_log),
        1,
        "remote install should still perform exactly one discovery clone, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fetch_count(&fetch_log),
        1,
        "remote install should batch selected skills into one repo-cache fetch, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn tm_p295_032_apply_reports_skipped_repo_without_fetching_unchanged_lock_entries() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_mode_a_origin_repo(temp.path(), "apply-skip-origin");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_mode_a_config(
        temp.path(),
        &common::path_to_file_url(&repo_dir),
        &storage_root,
        &target_root,
    );

    let first_apply = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["apply", "--config"])
        .arg(&config_path)
        .output()
        .expect("run initial apply");
    assert_eq!(
        first_apply.status.code(),
        Some(0),
        "initial apply should succeed, stderr={}",
        String::from_utf8_lossy(&first_apply.stderr)
    );

    let fetch_log = temp.path().join("git-fetches.log");
    let second_apply = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env("EDEN_SKILLS_TEST_GIT_FETCH_LOG", &fetch_log)
        .args(["apply", "--config"])
        .arg(&config_path)
        .output()
        .expect("run repeated apply");
    assert_eq!(
        second_apply.status.code(),
        Some(0),
        "repeated apply should succeed, stderr={}",
        String::from_utf8_lossy(&second_apply.stderr)
    );

    let stdout = String::from_utf8_lossy(&second_apply.stdout);
    assert_eq!(
        fetch_count(&fetch_log),
        0,
        "apply should skip network fetch for unchanged lock entries, stdout={stdout}"
    );
    assert!(
        stdout.contains("1 skipped"),
        "apply should report one skipped repo-sync task for unchanged lock entries, stdout={stdout}"
    );
}

#[test]
fn tm_p295_033_repair_always_fetches_repos_even_when_lock_entries_are_unchanged() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_mode_a_origin_repo(temp.path(), "repair-sync-origin");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_mode_a_config(
        temp.path(),
        &common::path_to_file_url(&repo_dir),
        &storage_root,
        &target_root,
    );

    let first_apply = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["apply", "--config"])
        .arg(&config_path)
        .output()
        .expect("run initial apply");
    assert_eq!(
        first_apply.status.code(),
        Some(0),
        "initial apply should succeed, stderr={}",
        String::from_utf8_lossy(&first_apply.stderr)
    );

    let fetch_log = temp.path().join("git-fetches.log");
    let repair_output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env("EDEN_SKILLS_TEST_GIT_FETCH_LOG", &fetch_log)
        .args(["repair", "--config"])
        .arg(&config_path)
        .output()
        .expect("run repair");
    assert_eq!(
        repair_output.status.code(),
        Some(0),
        "repair should succeed, stderr={}",
        String::from_utf8_lossy(&repair_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&repair_output.stdout);
    assert_eq!(
        fetch_count(&fetch_log),
        1,
        "repair should always fetch unchanged repos instead of skipping them, stdout={stdout}"
    );
    assert!(
        stdout.contains("1 skipped"),
        "repair should still report a skipped sync outcome after fetch when HEAD is unchanged, stdout={stdout}"
    );
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

    common::run_git_cmd(&repo, &["init"]);
    common::run_git_cmd(&repo, &["config", "user.email", "test@example.com"]);
    common::run_git_cmd(&repo, &["config", "user.name", "eden-skills-test"]);
    common::run_git_cmd(&repo, &["add", "."]);
    common::run_git_cmd(&repo, &["commit", "-m", "init"]);
    common::run_git_cmd(&repo, &["branch", "-M", "main"]);
    repo
}

fn init_remote_multi_skill_repo(base: &Path, repo_name: &str, skill_names: &[&str]) -> PathBuf {
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
description: Remote skill
---
"#
            ),
        )
        .expect("write skill");
        fs::write(skill_dir.join("README.md"), format!("{skill_name}\n")).expect("write readme");
    }

    common::run_git_cmd(&repo, &["init"]);
    common::run_git_cmd(&repo, &["config", "user.email", "test@example.com"]);
    common::run_git_cmd(&repo, &["config", "user.name", "eden-skills-test"]);
    common::run_git_cmd(&repo, &["add", "."]);
    common::run_git_cmd(&repo, &["commit", "-m", "init"]);
    common::run_git_cmd(&repo, &["branch", "-M", "main"]);
    repo
}

fn init_mode_a_origin_repo(base: &Path, repo_name: &str) -> PathBuf {
    let repo = base.join(repo_name);
    fs::create_dir_all(repo.join("packages").join("browser")).expect("create repo tree");
    fs::write(
        repo.join("packages").join("browser").join("README.txt"),
        "seed\n",
    )
    .expect("write source file");

    common::run_git_cmd(&repo, &["init"]);
    common::run_git_cmd(&repo, &["config", "user.email", "test@example.com"]);
    common::run_git_cmd(&repo, &["config", "user.name", "eden-skills-test"]);
    common::run_git_cmd(&repo, &["add", "."]);
    common::run_git_cmd(&repo, &["commit", "-m", "init"]);
    common::run_git_cmd(&repo, &["branch", "-M", "main"]);
    repo
}

fn write_mode_a_config(
    base: &Path,
    repo_url: &str,
    storage_root: &Path,
    target_root: &Path,
) -> PathBuf {
    let config_path = base.join("skills.toml");
    let config = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n\n[[skills]]\nid = \"demo-skill\"\n\n[skills.source]\nrepo = \"{}\"\nsubpath = \"packages/browser\"\nref = \"main\"\n\n[skills.install]\nmode = \"symlink\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{}\"\n\n[skills.verify]\nenabled = true\nchecks = [\"path-exists\", \"target-resolves\", \"is-symlink\"]\n\n[skills.safety]\nno_exec_metadata_only = false\n",
        common::toml_escape_path(storage_root),
        common::toml_escape_string(repo_url),
        common::toml_escape_path(target_root)
    );
    fs::write(&config_path, config).expect("write mode A config");
    config_path
}

fn clone_count(log_path: &Path) -> usize {
    event_count(log_path, "clone")
}

fn fetch_count(log_path: &Path) -> usize {
    event_count(log_path, "fetch")
}

fn event_count(log_path: &Path, event: &str) -> usize {
    fs::read_to_string(log_path)
        .unwrap_or_default()
        .lines()
        .filter(|line| *line == event)
        .count()
}

