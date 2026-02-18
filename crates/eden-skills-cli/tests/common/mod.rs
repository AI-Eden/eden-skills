#![allow(dead_code)]

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use eden_skills_cli::commands::CommandOptions;

pub const SKILL_ID: &str = "demo-skill";

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
    storage_root.join(SKILL_ID).join("packages").join("browser")
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
    run_icacls(path, &["/inheritance:r", "/grant:r", "*S-1-1-0:(OI)(CI)RX"]);
    original
}

#[cfg(windows)]
pub fn restore_permissions(path: &Path, _permissions: fs::Permissions) {
    run_icacls(path, &["/reset", "/T", "/C"]);
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
