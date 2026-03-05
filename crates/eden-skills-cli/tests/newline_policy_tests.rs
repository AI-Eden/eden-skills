use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::tempdir;

#[test]
fn tm_p29_036_error_without_hint_has_no_trailing_blank_line() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");

    let output = eden_command(&home_dir)
        .args(["--color", "never", "set", "demo-skill", "--config"])
        .arg(&config_path)
        .output()
        .expect("run set without mutation flags");

    assert_eq!(
        output.status.code(),
        Some(2),
        "set without mutation flags must be invalid arguments, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("set requires at least one mutation flag"),
        "expected explicit validation message, stderr={stderr}"
    );
    assert!(
        !stderr.ends_with("\n\n"),
        "error without hint must not end with a blank line, stderr={stderr:?}"
    );
}

#[test]
fn tm_p29_037_error_with_hint_has_single_separator_blank_line() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let missing_config = temp.path().join("missing").join("skills.toml");

    let output = eden_command(&home_dir)
        .args(["--color", "never", "list", "--config"])
        .arg(&missing_config)
        .output()
        .expect("run list with missing config");

    assert_eq!(
        output.status.code(),
        Some(1),
        "list with missing config should be runtime error, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("\n\n  ~> "),
        "error with hint must include exactly one blank separator before hint, stderr={stderr:?}"
    );
    assert!(
        !stderr.contains("\n\n\n"),
        "error output must not contain multiple consecutive blank lines, stderr={stderr:?}"
    );
    assert!(
        !stderr.ends_with("\n\n"),
        "error with hint must not end with a trailing blank line, stderr={stderr:?}"
    );
}

#[test]
fn tm_p29_038_clap_error_has_no_trailing_blank_lines() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");

    let output = eden_command(&home_dir)
        .args(["--color", "never", "lis"])
        .output()
        .expect("run unknown subcommand");

    assert_eq!(
        output.status.code(),
        Some(2),
        "unknown subcommand should return invalid arguments (2), stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unrecognized subcommand"),
        "clap parsing failure should mention unrecognized subcommand, stderr={stderr}"
    );
    assert!(
        !stderr.contains("\n\n\n"),
        "clap-derived error output must not include triple newlines, stderr={stderr:?}"
    );
    assert!(
        !stderr.ends_with("\n\n"),
        "clap-derived error must not end with blank line, stderr={stderr:?}"
    );
}

#[test]
fn tm_p29_039_list_doctor_plan_outputs_end_without_trailing_blank_lines() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");

    let init_output = eden_command(&home_dir)
        .args(["--color", "never", "init", "--force", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    assert_success(&init_output, "init");

    for subcommand in ["list", "doctor", "plan"] {
        let output = eden_command(&home_dir)
            .args(["--color", "never", subcommand, "--config"])
            .arg(&config_path)
            .output()
            .unwrap_or_else(|err| panic!("run {subcommand}: {err}"));
        assert_success(&output, subcommand);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.ends_with("\n\n"),
            "{subcommand} output must not end with a blank line, stdout={stdout:?}"
        );
    }
}

#[test]
fn tm_p29_040_install_remove_update_outputs_end_without_trailing_blank_lines() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let install_repo = setup_local_discovery_repo(temp.path(), "install-source", "demo-skill");

    let install_output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["--color", "never", "install"])
        .arg(path_as_relative_arg(&install_repo))
        .args(["--all", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_success(&install_output, "install");
    let install_stdout = String::from_utf8_lossy(&install_output.stdout);
    assert!(
        !install_stdout.ends_with("\n\n"),
        "install output must not end with a blank line, stdout={install_stdout:?}"
    );

    let remove_output = eden_command(&home_dir)
        .args([
            "--color",
            "never",
            "remove",
            "demo-skill",
            "--yes",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run remove");
    assert_success(&remove_output, "remove");
    let remove_stdout = String::from_utf8_lossy(&remove_output.stdout);
    assert!(
        !remove_stdout.ends_with("\n\n"),
        "remove output must not end with a blank line, stdout={remove_stdout:?}"
    );

    let update_config_path = temp.path().join("update-skills.toml");
    let registry_repo = init_git_repo(
        temp.path(),
        "registry-official",
        &[("manifest.toml", "format_version = 1\nname = \"official\"\n")],
    );
    write_registry_config(&update_config_path, temp.path(), &registry_repo);

    let update_output = eden_command(&home_dir)
        .args(["--color", "never", "update", "--config"])
        .arg(&update_config_path)
        .output()
        .expect("run update");
    assert_success(&update_output, "update");
    let update_stdout = String::from_utf8_lossy(&update_output.stdout);
    assert!(
        !update_stdout.ends_with("\n\n"),
        "update output must not end with a blank line, stdout={update_stdout:?}"
    );
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command
        .env("HOME", home_dir)
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env_remove("EDEN_SKILLS_FORCE_TTY")
        .env_remove("EDEN_SKILLS_TEST_CONFIRM")
        .env_remove("EDEN_SKILLS_TEST_SKILL_INPUT")
        .env_remove("EDEN_SKILLS_TEST_REMOVE_INPUT");
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

fn setup_local_discovery_repo(base: &Path, name: &str, skill_name: &str) -> PathBuf {
    let repo_dir = base.join(name);
    let skill_dir = repo_dir.join("skills").join(skill_name);
    fs::create_dir_all(&skill_dir).expect("create local discovery skill dir");
    fs::write(
        skill_dir.join("SKILL.md"),
        format!("---\nname: {skill_name}\ndescription: demo\n---\n"),
    )
    .expect("write SKILL.md");
    fs::write(skill_dir.join("README.md"), "demo\n").expect("write README.md");
    repo_dir
}

fn path_as_relative_arg(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|segment| segment.to_str())
        .expect("path should have UTF-8 file name");
    format!("./{name}")
}

fn write_registry_config(config_path: &Path, base: &Path, registry_repo: &Path) {
    let storage_root = base.join("storage");
    let config = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n\n[registries]\nofficial = {{ url = \"{}\", priority = 100 }}\n",
        toml_escape_path(&storage_root),
        toml_escape_str(&as_file_url(registry_repo)),
    );
    fs::write(config_path, config).expect("write update config");
}

fn init_git_repo(base: &Path, name: &str, files: &[(&str, &str)]) -> PathBuf {
    let repo_dir = base.join(name);
    for (relative, content) in files {
        let path = repo_dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(path, content).expect("write repository file");
    }

    run_git(&repo_dir, &["init"]);
    run_git(&repo_dir, &["config", "user.email", "test@example.com"]);
    run_git(&repo_dir, &["config", "user.name", "eden-skills-test"]);
    run_git(&repo_dir, &["add", "."]);
    run_git(&repo_dir, &["commit", "-m", "init"]);
    run_git(&repo_dir, &["branch", "-M", "main"]);
    repo_dir
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

fn assert_success(output: &Output, label: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{label} should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
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
    value.replace('\\', "\\\\").replace('\"', "\\\"")
}
