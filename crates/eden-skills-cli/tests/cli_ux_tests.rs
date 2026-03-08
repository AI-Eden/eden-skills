mod common;

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

#[test]
fn tty_install_output_contains_ansi_color_and_status_symbols() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_local_skill_repo(temp.path(), "tty-skill", "tty-skill");
    let config_path = temp.path().join("skills.toml");

    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .args(["install", &path_as_relative_arg(&repo_dir), "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        has_ansi_codes(&stdout),
        "expected ANSI codes in TTY mode, stdout={stdout}"
    );
    assert!(
        stdout.contains('✓') || stdout.contains('·') || stdout.contains('!'),
        "expected status symbols in TTY mode, stdout={stdout}"
    );
}

#[test]
fn no_color_disables_ansi_but_keeps_functional_status_output() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_local_skill_repo(temp.path(), "no-color-skill", "no-color-skill");
    let config_path = temp.path().join("skills.toml");

    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("NO_COLOR", "1")
        .args(["install", &path_as_relative_arg(&repo_dir), "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !has_ansi_codes(&stdout),
        "NO_COLOR should disable ANSI escapes, stdout={stdout}"
    );
    assert!(
        stdout.contains("install") && (stdout.contains('✓') || stdout.contains("status=installed")),
        "functional install status should remain visible, stdout={stdout}"
    );
}

#[test]
fn force_color_enables_ansi_even_on_non_tty() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_local_skill_repo(temp.path(), "force-color-skill", "force-color-skill");
    let config_path = temp.path().join("skills.toml");

    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("CI")
        .env("FORCE_COLOR", "1")
        .args(["install", &path_as_relative_arg(&repo_dir), "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        has_ansi_codes(&stdout),
        "FORCE_COLOR should enable ANSI on non-TTY output, stdout={stdout}"
    );
}

#[test]
fn ci_env_disables_ansi_even_when_tty_is_forced() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_local_skill_repo(temp.path(), "ci-skill", "ci-skill");
    let config_path = temp.path().join("skills.toml");

    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("CI", "1")
        .args(["install", &path_as_relative_arg(&repo_dir), "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !has_ansi_codes(&stdout),
        "CI should disable ANSI output, stdout={stdout}"
    );
}

#[test]
fn install_json_output_keeps_contract_and_omits_visual_elements() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_local_skill_repo(temp.path(), "json-skill-repo", "json-skill");
    let config_path = temp.path().join("skills.toml");

    let init_output = common::eden_command(&home_dir)
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    common::assert_success(&init_output);

    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .args([
            "install",
            &path_as_relative_arg(&repo_dir),
            "--json",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install --json");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !has_ansi_codes(&stdout),
        "--json output must not contain ANSI escapes, stdout={stdout}"
    );
    assert!(
        !stdout.contains('✓')
            && !stdout.contains('✗')
            && !stdout.contains('·')
            && !stdout.contains('!'),
        "--json output must not contain visual symbols, stdout={stdout}"
    );

    let payload: serde_json::Value =
        serde_json::from_str(&stdout).expect("install --json must emit valid JSON");
    let object = payload
        .as_object()
        .expect("install payload should be an object");
    assert_eq!(
        object.len(),
        2,
        "install JSON contract should remain a 2-field object: skills + status, payload={payload}"
    );
    assert_eq!(
        payload.get("status").and_then(|value| value.as_str()),
        Some("installed")
    );
    let skills = payload
        .get("skills")
        .and_then(|value| value.as_array())
        .expect("install JSON should include `skills` array");
    assert_eq!(skills.len(), 1, "payload={payload}");
    assert_eq!(skills[0].as_str(), Some("json-skill"));
}

#[test]
fn tty_remote_install_clone_phase_shows_spinner_and_completion_status() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_git_skill_repo(temp.path(), "remote-spinner-repo", "remote-spinner-skill");
    let source = common::path_to_file_url(&repo_dir);
    let config_path = temp.path().join("skills.toml");

    let output = common::eden_command(&home_dir)
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .args(["install", &source, "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Cloning"),
        "TTY remote install should show clone phase spinner/action line, stdout={stdout}"
    );
    assert!(
        stdout.contains("done") || stdout.contains('✓'),
        "TTY remote install should show completion status after clone, stdout={stdout}"
    );
}

#[test]
fn non_tty_remote_install_disables_spinner_output() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_git_skill_repo(temp.path(), "remote-non-tty-repo", "remote-non-tty-skill");
    let source = common::path_to_file_url(&repo_dir);
    let config_path = temp.path().join("skills.toml");

    let output = common::eden_command(&home_dir)
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env_remove("EDEN_SKILLS_FORCE_TTY")
        .args(["install", &source, "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Cloning"),
        "non-TTY output should not render spinner/action clone line, stdout={stdout}"
    );
    assert!(
        !has_ansi_codes(&stdout),
        "non-TTY output should not contain ANSI escapes, stdout={stdout}"
    );
}

fn init_local_skill_repo(base: &Path, name: &str, skill_name: &str) -> PathBuf {
    let repo_dir = base.join(name);
    fs::create_dir_all(&repo_dir).expect("create local skill repo");
    fs::write(
        repo_dir.join("SKILL.md"),
        format!("---\nname: {skill_name}\ndescription: demo\n---\n"),
    )
    .expect("write SKILL.md");
    fs::write(repo_dir.join("README.md"), "demo").expect("write readme");
    repo_dir
}

fn init_git_skill_repo(base: &Path, name: &str, skill_name: &str) -> PathBuf {
    let repo_dir = init_local_skill_repo(base, name, skill_name);
    common::run_git_cmd(&repo_dir, &["init"]);
    common::run_git_cmd(&repo_dir, &["config", "user.email", common::TEST_GIT_EMAIL]);
    common::run_git_cmd(&repo_dir, &["config", "user.name", common::TEST_GIT_NAME]);
    common::run_git_cmd(&repo_dir, &["add", "."]);
    common::run_git_cmd(&repo_dir, &["commit", "-m", "init"]);
    common::run_git_cmd(&repo_dir, &["branch", "-M", "main"]);
    repo_dir
}

fn has_ansi_codes(text: &str) -> bool {
    text.as_bytes().windows(2).any(|window| window == b"\x1b[")
}

fn path_as_relative_arg(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .expect("path should have valid UTF-8 file name");
    format!("./{file_name}")
}
