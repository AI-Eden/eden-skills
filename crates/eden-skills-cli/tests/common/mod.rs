#![allow(dead_code)]

use std::fs;
#[cfg(windows)]
use std::io::ErrorKind;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use eden_skills_cli::commands::CommandOptions;

pub const SKILL_ID: &str = "demo-skill";

pub const TEST_GIT_EMAIL: &str = "test@example.com";
pub const TEST_GIT_NAME: &str = "eden-skills-test";

pub fn default_options() -> CommandOptions {
    CommandOptions {
        strict: false,
        json: false,
    }
}

pub fn init_origin_repo(base: &Path) -> PathBuf {
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

pub fn write_config(
    base: &Path,
    repo_url: &str,
    install_mode: &str,
    verify_checks: &[&str],
    storage_root: &Path,
    target_root: &Path,
) -> PathBuf {
    write_config_with_safety(
        base,
        repo_url,
        install_mode,
        verify_checks,
        storage_root,
        target_root,
        false,
    )
}

pub fn write_config_with_safety(
    base: &Path,
    repo_url: &str,
    install_mode: &str,
    verify_checks: &[&str],
    storage_root: &Path,
    target_root: &Path,
    no_exec_metadata_only: bool,
) -> PathBuf {
    let checks = verify_checks
        .iter()
        .map(|check| format!("\"{check}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let config = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n\n[[skills]]\nid = \"{}\"\n\n[skills.source]\nrepo = \"{}\"\nsubpath = \"packages/browser\"\nref = \"main\"\n\n[skills.install]\nmode = \"{}\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{}\"\n\n[skills.verify]\nenabled = true\nchecks = [{}]\n\n[skills.safety]\nno_exec_metadata_only = {}\n",
        toml_escape(storage_root),
        SKILL_ID,
        toml_escape_str(repo_url),
        install_mode,
        toml_escape(target_root),
        checks,
        no_exec_metadata_only
    );
    let config_path = base.join("skills.toml");
    fs::write(&config_path, config).expect("write config");
    config_path
}

pub fn expected_source_path(storage_root: &Path) -> PathBuf {
    let repo_cache_root = storage_root.join(".repos");
    if let Ok(entries) = fs::read_dir(&repo_cache_root) {
        let mut cache_dirs = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        cache_dirs.sort();
        if let [cache_dir] = cache_dirs.as_slice() {
            return cache_dir.join("packages").join("browser");
        }
    }
    storage_root.join(SKILL_ID).join("packages").join("browser")
}

pub fn expected_safety_metadata_path(storage_root: &Path) -> PathBuf {
    let repo_cache_root = storage_root.join(".repos");
    if let Ok(entries) = fs::read_dir(&repo_cache_root) {
        let mut cache_dirs = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        cache_dirs.sort();
        if let [cache_dir] = cache_dirs.as_slice() {
            return cache_dir.join(".eden-safety.toml");
        }
    }
    storage_root.join(SKILL_ID).join(".eden-safety.toml")
}

pub fn expected_target_path(target_root: &Path) -> PathBuf {
    target_root.join(SKILL_ID)
}

pub fn as_file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

pub fn resolved_symlink(path: &Path) -> PathBuf {
    let raw = fs::read_link(path).expect("read symlink");
    if raw.is_absolute() {
        raw
    } else {
        path.parent()
            .expect("parent")
            .join(raw)
            .canonicalize()
            .expect("canonicalize")
    }
}

pub fn assert_paths_resolve_to_same_location(expected: &Path, actual: &Path) {
    let expected_canonical = fs::canonicalize(expected).unwrap_or_else(|err| {
        panic!(
            "failed to canonicalize expected path `{}`: {err}",
            expected.display()
        )
    });
    let actual_canonical = fs::canonicalize(actual).unwrap_or_else(|err| {
        panic!(
            "failed to canonicalize actual path `{}`: {err}",
            actual.display()
        )
    });
    assert_eq!(expected_canonical, actual_canonical);
}

#[cfg(unix)]
pub fn make_read_only_dir(path: &Path) -> fs::Permissions {
    fs::create_dir_all(path).expect("create restricted directory");
    let original = fs::metadata(path)
        .expect("restricted metadata")
        .permissions();
    let mut read_exec_only = original.clone();
    read_exec_only.set_mode(0o555);
    fs::set_permissions(path, read_exec_only).expect("set read-only permissions");
    original
}

#[cfg(unix)]
pub fn restore_permissions(path: &Path, permissions: fs::Permissions) {
    fs::set_permissions(path, permissions).expect("restore original permissions");
}

#[cfg(windows)]
pub fn make_read_only_dir(path: &Path) -> fs::Permissions {
    fs::create_dir_all(path).expect("create restricted directory");
    let original = fs::metadata(path)
        .expect("restricted metadata")
        .permissions();
    let principal = current_windows_principal();
    let grant_rule_self = format!("{principal}:RX");
    let grant_rule_children = format!("{principal}:(OI)(CI)RX");
    let deny_rule_self = format!("{principal}:W");
    let deny_rule_children = format!("{principal}:(OI)(CI)W");
    run_icacls(path, &["/inheritance:r"]);
    run_icacls(path, &["/grant:r", &grant_rule_self]);
    run_icacls(path, &["/grant", &grant_rule_children]);
    run_icacls(path, &["/deny", &deny_rule_self]);
    run_icacls(path, &["/deny", &deny_rule_children]);
    original
}

#[cfg(windows)]
pub fn restore_permissions(path: &Path, _permissions: fs::Permissions) {
    run_icacls(path, &["/reset", "/T", "/C"]);
}

pub fn remove_symlink(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        fs::remove_file(path)
    }

    #[cfg(windows)]
    {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == ErrorKind::PermissionDenied => fs::remove_dir(path),
            Err(err) => Err(err),
        }
    }
}

#[cfg(unix)]
pub fn create_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
pub fn create_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(source, target)
}

#[cfg(windows)]
fn run_icacls(path: &Path, args: &[&str]) {
    let output = Command::new("icacls")
        .arg(path)
        .args(args)
        .output()
        .expect("spawn icacls");
    if output.status.success() {
        return;
    }
    panic!(
        "icacls {:?} failed for {}: status={} stderr=`{}` stdout=`{}`",
        args,
        path.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr).trim(),
        String::from_utf8_lossy(&output.stdout).trim()
    );
}

#[cfg(windows)]
fn current_windows_principal() -> String {
    let output = Command::new("whoami").output().expect("spawn whoami");
    if !output.status.success() {
        panic!(
            "whoami failed: status={} stderr=`{}` stdout=`{}`",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim(),
            String::from_utf8_lossy(&output.stdout).trim()
        );
    }
    String::from_utf8_lossy(&output.stdout).trim().to_string()
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

pub fn run_git_cmd(cwd: &Path, args: &[&str]) {
    run_git(cwd, args);
}

fn toml_escape(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

fn toml_escape_str(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

// ---- unified helpers added during WP-2 (Phase 2.99 Code Health) ----

/// Build a `Command` for the eden-skills binary with `HOME` / `USERPROFILE`
/// pointed at the given directory to isolate tests from the real home.
pub fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

/// Assert that a command completed with exit code 0, printing stderr on
/// failure.
pub fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "command should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Assert that a command completed with exit code 0, with a custom label
/// in the failure message.
pub fn assert_success_labeled(output: &Output, label: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{label} should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Convert a filesystem path to a `file://` URL, handling Windows drive
/// letter normalization.
pub fn path_to_file_url(path: &Path) -> String {
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

/// Escape a filesystem path for embedding inside a TOML double-quoted string.
pub fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

/// Escape an arbitrary string for embedding inside a TOML double-quoted string.
pub fn toml_escape_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Initialize a bare-bones git repo with a single commit.  `files` is a
/// list of `(relative_path, content)` pairs written before the initial
/// commit.
pub fn init_git_repo(base: &Path, name: &str, files: &[(&str, &str)]) -> PathBuf {
    let repo = base.join(name);
    for (rel, content) in files {
        let file_path = repo.join(rel);
        fs::create_dir_all(file_path.parent().expect("parent")).expect("create parent dirs");
        fs::write(&file_path, content).expect("write file");
    }
    run_git_cmd(&repo, &["init"]);
    run_git_cmd(&repo, &["config", "user.email", TEST_GIT_EMAIL]);
    run_git_cmd(&repo, &["config", "user.name", TEST_GIT_NAME]);
    run_git_cmd(&repo, &["add", "."]);
    run_git_cmd(&repo, &["commit", "-m", "init"]);
    run_git_cmd(&repo, &["branch", "-M", "main"]);
    repo
}
