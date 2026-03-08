mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    common::assert_success_labeled(&init_output, "init");

    for subcommand in ["list", "doctor", "plan"] {
        let output = eden_command(&home_dir)
            .args(["--color", "never", subcommand, "--config"])
            .arg(&config_path)
            .output()
            .unwrap_or_else(|err| panic!("run {subcommand}: {err}"));
        common::assert_success_labeled(&output, subcommand);
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
    common::assert_success_labeled(&install_output, "install");
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
    common::assert_success_labeled(&remove_output, "remove");
    let remove_stdout = String::from_utf8_lossy(&remove_output.stdout);
    assert!(
        !remove_stdout.ends_with("\n\n"),
        "remove output must not end with a blank line, stdout={remove_stdout:?}"
    );

    let update_config_path = temp.path().join("update-skills.toml");
    let registry_repo = common::init_git_repo(
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
    common::assert_success_labeled(&update_output, "update");
    let update_stdout = String::from_utf8_lossy(&update_output.stdout);
    assert!(
        !update_stdout.ends_with("\n\n"),
        "update output must not end with a blank line, stdout={update_stdout:?}"
    );
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = common::eden_command(home_dir);
    command
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env_remove("EDEN_SKILLS_FORCE_TTY")
        .env_remove("EDEN_SKILLS_TEST_CONFIRM")
        .env_remove("EDEN_SKILLS_TEST_SKILL_INPUT")
        .env_remove("EDEN_SKILLS_TEST_REMOVE_INPUT");
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
        common::toml_escape_path(&storage_root),
        common::toml_escape_string(&common::path_to_file_url(registry_repo)),
    );
    fs::write(config_path, config).expect("write update config");
}
