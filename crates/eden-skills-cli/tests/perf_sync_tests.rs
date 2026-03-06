mod common;

use std::env;
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
    let git_probe = create_git_probe(temp.path());
    let repo_url = as_file_url(&repo_dir);

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("PATH", &git_probe.path_env)
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
        count_git_subcommand(&git_probe.log_path, "clone"),
        1,
        "remote install should still perform exactly one discovery clone, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        count_git_subcommand(&git_probe.log_path, "fetch"),
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
        &as_file_url(&repo_dir),
        &storage_root,
        &target_root,
    );

    let first_apply = eden_command(&home_dir)
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

    let git_probe = create_git_probe(temp.path());
    let second_apply = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("PATH", &git_probe.path_env)
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
        count_git_subcommand(&git_probe.log_path, "fetch"),
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
        &as_file_url(&repo_dir),
        &storage_root,
        &target_root,
    );

    let first_apply = eden_command(&home_dir)
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

    let git_probe = create_git_probe(temp.path());
    let repair_output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("PATH", &git_probe.path_env)
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
        count_git_subcommand(&git_probe.log_path, "fetch"),
        1,
        "repair should always fetch unchanged repos instead of skipping them, stdout={stdout}"
    );
    assert!(
        stdout.contains("1 skipped"),
        "repair should still report a skipped sync outcome after fetch when HEAD is unchanged, stdout={stdout}"
    );
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

struct GitProbe {
    log_path: PathBuf,
    path_env: OsString,
}

fn create_git_probe(base: &Path) -> GitProbe {
    let wrapper_dir = base.join("git-probe-bin");
    fs::create_dir_all(&wrapper_dir).expect("create git probe dir");
    let log_path = base.join("git-subcommands.log");
    let real_git = find_git_binary();

    #[cfg(unix)]
    {
        let wrapper_path = wrapper_dir.join("git");
        fs::write(
            &wrapper_path,
            format!(
                "#!/bin/sh\nsubcmd=\"\"\nskip_next=0\nfor arg in \"$@\"; do\n  if [ \"$skip_next\" -eq 1 ]; then\n    skip_next=0\n    continue\n  fi\n  case \"$arg\" in\n    -C|--git-dir|--work-tree|--namespace)\n      skip_next=1\n      ;;\n    -*)\n      ;;\n    *)\n      subcmd=\"$arg\"\n      break\n      ;;\n  esac\ndone\nprintf '%s\\n' \"$subcmd\" >> \"{}\"\nexec \"{}\" \"$@\"\n",
                log_path.display(),
                real_git.display()
            ),
        )
        .expect("write unix git probe");
        let mut permissions = fs::metadata(&wrapper_path)
            .expect("wrapper metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&wrapper_path, permissions).expect("chmod git probe");
    }

    #[cfg(windows)]
    {
        let wrapper_path = wrapper_dir.join("git.cmd");
        fs::write(
            &wrapper_path,
            format!(
                "@echo off\r\nsetlocal EnableDelayedExpansion\r\nset \"skip_next=0\"\r\nset \"subcmd=\"\r\nfor %%A in (%*) do (\r\n  if \"!skip_next!\"==\"1\" (\r\n    set \"skip_next=0\"\r\n  ) else if /I \"%%~A\"==\"-C\" (\r\n    set \"skip_next=1\"\r\n  ) else if /I \"%%~A\"==\"--git-dir\" (\r\n    set \"skip_next=1\"\r\n  ) else if /I \"%%~A\"==\"--work-tree\" (\r\n    set \"skip_next=1\"\r\n  ) else if /I \"%%~A\"==\"--namespace\" (\r\n    set \"skip_next=1\"\r\n  ) else if not defined subcmd (\r\n    if not \"%%~A\"==\"\" set \"subcmd=%%~A\"\r\n  )\r\n)\r\n>>\"{}\" echo !subcmd!\r\n\"{}\" %*\r\n",
                log_path.display(),
                real_git.display()
            ),
        )
        .expect("write windows git probe");
    }

    let path_env = prepend_path(&wrapper_dir, env::var_os("PATH"));
    GitProbe { log_path, path_env }
}

fn prepend_path(prefix: &Path, existing: Option<OsString>) -> OsString {
    let mut paths = vec![prefix.to_path_buf()];
    if let Some(existing) = existing {
        paths.extend(env::split_paths(&existing));
    }
    env::join_paths(paths).expect("join PATH")
}

fn find_git_binary() -> PathBuf {
    let path = env::var_os("PATH").expect("PATH should exist");
    for dir in env::split_paths(&path) {
        for candidate in ["git", "git.exe", "git.cmd", "git.bat"] {
            let path = dir.join(candidate);
            if path.is_file() {
                return path;
            }
        }
    }
    panic!("failed to locate git binary in PATH");
}

fn count_git_subcommand(log_path: &Path, subcommand: &str) -> usize {
    fs::read_to_string(log_path)
        .unwrap_or_default()
        .lines()
        .filter(|line| *line == subcommand)
        .count()
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

    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "test@example.com"]);
    run_git(&repo, &["config", "user.name", "eden-skills-test"]);
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "init"]);
    run_git(&repo, &["branch", "-M", "main"]);
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

    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "test@example.com"]);
    run_git(&repo, &["config", "user.name", "eden-skills-test"]);
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "init"]);
    run_git(&repo, &["branch", "-M", "main"]);
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
        toml_escape_path(storage_root),
        toml_escape_str(repo_url),
        toml_escape_path(target_root)
    );
    fs::write(&config_path, config).expect("write mode A config");
    config_path
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

fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

fn toml_escape_str(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
