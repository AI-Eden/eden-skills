use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use toml::Value;

#[test]
fn version_flag_and_short_alias_print_package_version() {
    let long = run_eden(["--version"]);
    assert_success(&long);
    let long_text = String::from_utf8_lossy(&long.stdout).trim().to_string();
    assert_eq!(long_text, expected_version_line());

    let short = run_eden(["-V"]);
    assert_success(&short);
    let short_text = String::from_utf8_lossy(&short.stdout).trim().to_string();
    assert_eq!(short_text, expected_version_line());
}

#[test]
fn root_help_contains_version_about_groups_and_examples() {
    let output = run_eden(["--help"]);
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(&expected_version_line()),
        "root help should include version header, stdout={stdout}"
    );
    assert!(
        stdout
            .contains("Deterministic skill installation and reconciliation for agent environments"),
        "root help should include about text, stdout={stdout}"
    );
    assert!(
        stdout.contains("Install & Update:"),
        "root help should include Install & Update group, stdout={stdout}"
    );
    assert!(
        stdout.contains("State Reconciliation:"),
        "root help should include State Reconciliation group, stdout={stdout}"
    );
    assert!(
        stdout.contains("Configuration:"),
        "root help should include Configuration group, stdout={stdout}"
    );
    assert!(
        stdout.contains("Examples:"),
        "root help should include quickstart examples, stdout={stdout}"
    );
    assert!(
        stdout.contains("Documentation: https://github.com/AI-Eden/eden-skills"),
        "root help should include documentation link, stdout={stdout}"
    );
}

#[test]
fn subcommands_include_normative_about_descriptions() {
    for (cmd, expected_phrase) in [
        ("plan", "Preview planned actions without making changes"),
        ("apply", "Reconcile installed state with configuration"),
        ("doctor", "Diagnose configuration and installation health"),
        ("repair", "Auto-repair drifted or broken installations"),
        ("update", "Refresh registry sources to latest versions"),
        ("install", "Install skills from a URL, path, or registry"),
        ("init", "Create a new skills.toml configuration file"),
        ("list", "List configured skills and their targets"),
        ("add", "Add a skill entry to skills.toml"),
        ("remove", "Uninstall a skill and clean up its files"),
        ("set", "Modify properties of an existing skill entry"),
        ("config", "Export or import configuration"),
    ] {
        let output = run_eden([cmd, "--help"]);
        assert_success(&output);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(expected_phrase),
            "help for `{cmd}` should contain `{expected_phrase}`, stdout={stdout}"
        );
    }

    let export_help = run_eden(["config", "export", "--help"]);
    assert_success(&export_help);
    let export_stdout = String::from_utf8_lossy(&export_help.stdout);
    assert!(
        export_stdout.contains("Export configuration to stdout"),
        "config export help should include command about, stdout={export_stdout}"
    );

    let import_help = run_eden(["config", "import", "--help"]);
    assert_success(&import_help);
    let import_stdout = String::from_utf8_lossy(&import_help.stdout);
    assert!(
        import_stdout.contains("Import configuration from another file"),
        "config import help should include command about, stdout={import_stdout}"
    );
}

#[test]
fn install_help_shows_argument_descriptions() {
    let output = run_eden(["install", "--help"]);
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);

    for expected in [
        "URL, local path, or registry skill name",
        "Override the auto-derived skill identifier",
        "Git reference (branch, tag, or commit)",
        "Install only the named skill(s) from the repository",
        "Install all discovered skills without confirmation",
        "Skip all interactive confirmation prompts",
        "List discovered skills without installing",
        "Version constraint for registry mode (e.g. >=1.0)",
        "Use a specific registry for resolution",
        "Install to specific agent targets (e.g. claude-code, cursor)",
        "Preview what would be installed without making changes",
        "Use file copy instead of symlinks",
    ] {
        assert!(
            stdout.contains(expected),
            "install --help missing expected description `{expected}`, stdout={stdout}"
        );
    }
}

#[test]
fn short_flags_are_accepted_for_install_and_root_version() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let repo_dir = init_local_skill_repo(temp.path(), "short-flag-repo", "short-flag-skill");
    let source = repo_dir
        .to_str()
        .expect("local source path should be valid UTF-8")
        .to_string();

    let install_output = eden_command(&home_dir)
        .args([
            "install",
            &source,
            "-s",
            "short-flag-skill",
            "-t",
            "cursor",
            "-y",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install with short flags");
    assert_success(&install_output);

    let version_output = run_eden(["-V"]);
    assert_success(&version_output);
}

#[test]
fn install_copy_flag_persists_copy_mode_and_copy_verify_defaults() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let repo_dir = init_local_skill_repo(temp.path(), "copy-flag-repo", "copy-flag-skill");
    let source = repo_dir
        .to_str()
        .expect("local source path should be valid UTF-8")
        .to_string();

    let output = eden_command(&home_dir)
        .args(["install", &source, "--copy", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install --copy");
    assert_success(&output);

    let config_text = fs::read_to_string(&config_path).expect("read generated config");
    let parsed: Value = toml::from_str(&config_text).expect("parse generated config");
    let skills = parsed
        .get("skills")
        .and_then(Value::as_array)
        .expect("skills should be an array");
    assert_eq!(
        skills.len(),
        1,
        "expected one installed skill, config={config_text}"
    );

    let skill = skills[0].as_table().expect("skill entry should be a table");
    let install_mode = skill
        .get("install")
        .and_then(Value::as_table)
        .and_then(|table| table.get("mode"))
        .and_then(Value::as_str);
    assert_eq!(
        install_mode,
        Some("copy"),
        "install --copy should persist install.mode=copy, config={config_text}"
    );

    let verify_checks = skill
        .get("verify")
        .and_then(Value::as_table)
        .and_then(|table| table.get("checks"))
        .and_then(Value::as_array)
        .expect("verify.checks should be an array")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        verify_checks,
        vec!["path-exists", "content-present"],
        "copy mode should persist copy verify defaults, config={config_text}"
    );
}

fn expected_version_line() -> String {
    format!("eden-skills {}", env!("CARGO_PKG_VERSION"))
}

fn run_eden<const N: usize>(args: [&str; N]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(args)
        .output()
        .expect("run eden-skills")
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

fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "command should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
