use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn no_hardcoded_ansi_literals_in_ui_and_commands_sources() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources_to_check: Vec<PathBuf> = vec![crate_root.join("src/ui.rs")];
    let commands_dir = crate_root.join("src/commands");
    if commands_dir.is_dir() {
        for entry in fs::read_dir(&commands_dir).expect("read src/commands/") {
            let entry = entry.expect("read dir entry");
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                sources_to_check.push(path);
            }
        }
    } else {
        sources_to_check.push(crate_root.join("src/commands.rs"));
    }
    for path in &sources_to_check {
        let relative = path.strip_prefix(crate_root).unwrap_or(path);
        let source = fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", relative.display()));
        assert!(
            !source.contains("\\u{1b}[") && !source.contains("\\x1b["),
            "{} must not contain hardcoded ANSI literals",
            relative.display()
        );
    }
}

#[test]
fn console_crate_is_not_a_direct_cli_dependency() {
    let cargo_toml = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read crates/eden-skills-cli/Cargo.toml");
    assert!(
        !cargo_toml
            .lines()
            .any(|line| line.trim_start().starts_with("console")),
        "console must be removed as a direct dependency"
    );
}

#[test]
fn color_flag_auto_enables_on_tty_and_disables_on_non_tty() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_tty = temp.path().join("skills-tty.toml");
    let config_pipe = temp.path().join("skills-pipe.toml");
    let repo_dir = init_local_skill_repo(temp.path(), "auto-color-repo", "auto-color-skill");
    let source = path_as_relative_arg(&repo_dir);

    let tty_output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .args(["install", &source, "--color", "auto", "--config"])
        .arg(&config_tty)
        .output()
        .expect("run install --color auto in forced tty");
    assert_success(&tty_output);
    let tty_stdout = String::from_utf8_lossy(&tty_output.stdout);
    assert!(
        has_ansi_codes(&tty_stdout),
        "--color auto should emit ANSI in tty mode, stdout={tty_stdout}"
    );

    let non_tty_output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env_remove("EDEN_SKILLS_FORCE_TTY")
        .args(["install", &source, "--color", "auto", "--config"])
        .arg(&config_pipe)
        .output()
        .expect("run install --color auto in non-tty");
    assert_success(&non_tty_output);
    let non_tty_stdout = String::from_utf8_lossy(&non_tty_output.stdout);
    assert!(
        !has_ansi_codes(&non_tty_stdout),
        "--color auto should disable ANSI on non-tty, stdout={non_tty_stdout}"
    );
}

#[test]
fn color_flag_never_disables_ansi_even_when_tty_forced() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills-never.toml");
    let repo_dir = init_local_skill_repo(temp.path(), "never-color-repo", "never-color-skill");
    let source = path_as_relative_arg(&repo_dir);

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .args(["install", &source, "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install --color never");
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !has_ansi_codes(&stdout),
        "--color never must disable ANSI output, stdout={stdout}"
    );
}

#[test]
fn color_flag_always_enables_ansi_on_non_tty() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills-always.toml");
    let repo_dir = init_local_skill_repo(temp.path(), "always-color-repo", "always-color-skill");
    let source = path_as_relative_arg(&repo_dir);

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env_remove("EDEN_SKILLS_FORCE_TTY")
        .args(["install", &source, "--color", "always", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install --color always");
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        has_ansi_codes(&stdout),
        "--color always must force ANSI on non-tty output, stdout={stdout}"
    );
}

#[cfg(windows)]
#[test]
fn windows_color_always_enables_ansi_sequences() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills-windows-always.toml");
    let repo_dir = init_local_skill_repo(
        temp.path(),
        "windows-always-color-repo",
        "windows-always-color-skill",
    );
    let source = path_as_relative_arg(&repo_dir);

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env_remove("EDEN_SKILLS_FORCE_TTY")
        .args(["install", &source, "--color", "always", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install --color always on windows");
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        has_ansi_codes(&stdout),
        "--color always should emit ANSI on Windows terminals too, stdout={stdout}"
    );
}

#[test]
fn error_output_uses_error_prefix_and_hint_for_missing_config() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let missing_config = temp.path().join("does-not-exist").join("skills.toml");

    let output = eden_command(&home_dir)
        .args(["list", "--color", "always", "--config"])
        .arg(&missing_config)
        .output()
        .expect("run list with missing config");

    assert_eq!(
        output.status.code(),
        Some(1),
        "missing config should return runtime exit code 1, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error:"),
        "formatted error prefix is required, stderr={stderr}"
    );
    assert!(
        stderr.contains("~>") && !stderr.contains("hint:"),
        "hint line must use arrow prefix, stderr={stderr}"
    );
    assert!(
        stderr.contains(&format!(
            "config file not found: {}",
            missing_config.display()
        )),
        "missing-config message should include explicit path, stderr={stderr}"
    );
    assert!(
        stderr.contains("Run `eden-skills init` to create a new config."),
        "missing-config message should include remediation hint, stderr={stderr}"
    );
    assert!(
        has_ansi_codes(&stderr),
        "--color always should colorize the error prefix, stderr={stderr}"
    );
    assert!(
        !stderr.contains("io error: No such file or directory"),
        "raw io error text should be wrapped with contextual message, stderr={stderr}"
    );
}

#[test]
fn remove_unknown_skill_includes_available_skills_hint() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    write_single_skill_config(&config_path, temp.path(), "known-skill");

    let output = eden_command(&home_dir)
        .args(["remove", "missing-skill", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run remove with unknown skill id");

    assert_eq!(
        output.status.code(),
        Some(2),
        "unknown remove id should be argument error (exit code 2), stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("skill 'missing-skill' not found in config"),
        "message should explain unknown skill in config, stderr={stderr}"
    );
    assert!(
        stderr.contains("~> Available skills: known-skill"),
        "hint should list available skills with arrow prefix, stderr={stderr}"
    );
    assert!(
        !has_ansi_codes(&stderr),
        "--color never must disable ANSI in errors too, stderr={stderr}"
    );
}

#[test]
fn palette_avoids_truecolor_and_256color_sequences() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills-palette.toml");
    let repo_dir = init_local_skill_repo(temp.path(), "palette-repo", "palette-skill");
    let source = path_as_relative_arg(&repo_dir);

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .args(["install", &source, "--color", "always", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install with forced color");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        has_ansi_codes(&stdout),
        "expected ANSI output, stdout={stdout}"
    );
    assert!(
        !stdout.contains("38;2") && !stdout.contains("38;5"),
        "output should not use truecolor/256-color escapes, stdout={stdout}"
    );
}

#[test]
fn json_mode_ignores_color_always_and_emits_clean_json() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills-json.toml");
    let repo_dir = init_local_skill_repo(temp.path(), "json-color-repo", "json-color-skill");
    let source = path_as_relative_arg(&repo_dir);

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .args([
            "install", &source, "--json", "--color", "always", "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install --json --color always");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !has_ansi_codes(&stdout),
        "--json output must not include ANSI escapes, stdout={stdout}"
    );
    let payload: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|err| panic!("--json output must be valid JSON ({err}), stdout={stdout}"));
    assert_eq!(
        payload.get("status").and_then(Value::as_str),
        Some("installed")
    );
    assert_eq!(
        payload
            .get("skills")
            .and_then(Value::as_array)
            .map(|skills| skills.len()),
        Some(1),
        "install --json contract should remain stable, payload={payload}"
    );
}

#[test]
fn preflight_reports_missing_git_before_clone_attempt() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let repo_dir = init_git_skill_repo(temp.path(), "missing-git-repo", "missing-git-skill");
    let source = as_file_url(&repo_dir);
    let no_git_path = temp.path().join("no-git-bin");
    fs::create_dir_all(&no_git_path).expect("create no-git PATH directory");

    let output = eden_command(&home_dir)
        .env("PATH", &no_git_path)
        .args(["install", &source, "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install with git missing from PATH");

    assert_eq!(
        output.status.code(),
        Some(1),
        "missing git should return runtime exit code 1, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("git executable not found"),
        "missing-git preflight message should be explicit, stderr={stderr}"
    );
    assert!(
        stderr.contains("Install Git: https://git-scm.com/downloads"),
        "missing-git preflight should include install hint, stderr={stderr}"
    );
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

fn init_local_skill_repo(base: &Path, name: &str, skill_name: &str) -> PathBuf {
    let repo_dir = base.join(name);
    fs::create_dir_all(&repo_dir).expect("create local skill repo");
    fs::write(
        repo_dir.join("SKILL.md"),
        format!("---\nname: {skill_name}\ndescription: demo\n---\n"),
    )
    .expect("write SKILL.md");
    fs::write(repo_dir.join("README.md"), "demo").expect("write README.md");
    repo_dir
}

fn init_git_skill_repo(base: &Path, name: &str, skill_name: &str) -> PathBuf {
    let repo_dir = init_local_skill_repo(base, name, skill_name);
    run_git(&repo_dir, &["init"]);
    run_git(&repo_dir, &["config", "user.email", "test@example.com"]);
    run_git(&repo_dir, &["config", "user.name", "eden-skills-test"]);
    run_git(&repo_dir, &["add", "."]);
    run_git(&repo_dir, &["commit", "-m", "init"]);
    run_git(&repo_dir, &["branch", "-M", "main"]);
    repo_dir
}

fn write_single_skill_config(config_path: &Path, root: &Path, skill_id: &str) {
    let storage_root = root.join("storage");
    let config = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n\n[[skills]]\nid = \"{}\"\n\n[skills.source]\nrepo = \"https://example.com/repo.git\"\nsubpath = \".\"\nref = \"main\"\n\n[skills.install]\nmode = \"symlink\"\n\n[[skills.targets]]\nagent = \"claude-code\"\n\n[skills.verify]\nenabled = true\nchecks = [\"path-exists\", \"target-resolves\", \"is-symlink\"]\n\n[skills.safety]\nno_exec_metadata_only = false\n",
        toml_escape_path(&storage_root),
        skill_id
    );
    fs::write(config_path, config).expect("write test config");
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

fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "command should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
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
