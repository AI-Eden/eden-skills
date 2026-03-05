use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::{tempdir, TempDir};

struct Fixture {
    temp: TempDir,
    home_dir: PathBuf,
    config_path: PathBuf,
    target_roots: Vec<PathBuf>,
}

#[test]
fn tm_p28_018_doctor_header_styled() {
    let issue_fixture = setup_fixture(&["doctor-skill"], &["agent-a"], &["path-exists"], true);
    let issue_output = run_command_with_config(
        &issue_fixture,
        "never",
        true,
        &["doctor"],
        &[("EDEN_SKILLS_DOCKER_BIN", "docker")],
    );
    assert_success(&issue_output);

    let issue_stdout = String::from_utf8_lossy(&issue_output.stdout);
    assert!(
        issue_stdout.contains("Doctor") && issue_stdout.contains("issues detected"),
        "doctor output should contain styled header with issue count, stdout={issue_stdout}"
    );
    assert!(
        !issue_stdout.contains("doctor: detected"),
        "legacy doctor header format must be removed, stdout={issue_stdout}"
    );

    let clean_fixture = setup_fixture(&["healthy-skill"], &["agent-a"], &["path-exists"], true);
    let apply_output = run_command_with_config(
        &clean_fixture,
        "never",
        true,
        &["apply"],
        &[("EDEN_SKILLS_DOCKER_BIN", "docker")],
    );
    assert_success(&apply_output);
    let clean_output = run_command_with_config(
        &clean_fixture,
        "never",
        true,
        &["doctor"],
        &[("EDEN_SKILLS_DOCKER_BIN", "docker")],
    );
    assert_success(&clean_output);
    let clean_stdout = String::from_utf8_lossy(&clean_output.stdout);
    assert!(
        clean_stdout.contains("Doctor") && clean_stdout.contains("✓ no issues detected"),
        "doctor no-issue state should use styled success header, stdout={clean_stdout}"
    );
}

#[test]
fn tm_p28_019_doctor_findings_cards() {
    let fixture = setup_fixture(&["card-skill"], &["agent-a"], &["path-exists"], true);
    let output = run_command_with_config(
        &fixture,
        "never",
        true,
        &["doctor"],
        &[("EDEN_SKILLS_DOCKER_BIN", "docker")],
    );
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✗ [SOURCE_MISSING] card-skill"),
        "doctor finding card should include severity symbol, code, and skill id, stdout={stdout}"
    );
    assert!(
        stdout.contains("\n    ") && stdout.contains("\n    → "),
        "doctor finding card should include indented message and remediation lines, stdout={stdout}"
    );
    assert!(
        !stdout.contains("code=SOURCE_MISSING") && !stdout.contains("severity=error"),
        "legacy key=value doctor line format must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_020_doctor_summary_table_conditional() {
    let high_fixture = setup_fixture(
        &["table-skill"],
        &["agent-a", "agent-b", "agent-c", "agent-d"],
        &["path-exists"],
        true,
    );
    let high_apply = run_command_with_config(
        &high_fixture,
        "never",
        true,
        &["apply"],
        &[("EDEN_SKILLS_DOCKER_BIN", "docker")],
    );
    assert_success(&high_apply);
    for target_root in &high_fixture.target_roots {
        let target = target_root.join("table-skill");
        remove_symlink(&target).expect("remove target to create doctor finding");
    }
    let high_output = run_command_with_config(
        &high_fixture,
        "never",
        true,
        &["doctor"],
        &[("EDEN_SKILLS_DOCKER_BIN", "docker")],
    );
    assert_success(&high_output);
    let high_stdout = String::from_utf8_lossy(&high_output.stdout);
    assert!(
        high_stdout.contains("Sev")
            && high_stdout.contains("Code")
            && high_stdout.contains("Skill"),
        "doctor with >3 findings should include summary table header, stdout={high_stdout}"
    );

    let low_fixture = setup_fixture(
        &["table-skill"],
        &["agent-a", "agent-b"],
        &["path-exists"],
        true,
    );
    let low_apply = run_command_with_config(
        &low_fixture,
        "never",
        true,
        &["apply"],
        &[("EDEN_SKILLS_DOCKER_BIN", "docker")],
    );
    assert_success(&low_apply);
    for target_root in &low_fixture.target_roots {
        let target = target_root.join("table-skill");
        remove_symlink(&target).expect("remove target to create doctor finding");
    }
    let low_output = run_command_with_config(
        &low_fixture,
        "never",
        true,
        &["doctor"],
        &[("EDEN_SKILLS_DOCKER_BIN", "docker")],
    );
    assert_success(&low_output);
    let low_stdout = String::from_utf8_lossy(&low_output.stdout);
    assert!(
        !(low_stdout.contains("Sev")
            && low_stdout.contains("Code")
            && low_stdout.contains("Skill")),
        "doctor with <=3 findings should not include summary table, stdout={low_stdout}"
    );
}

#[test]
fn tm_p28_023_init_next_steps() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = home_dir.join(".eden-skills").join("skills.toml");

    let output = eden_command(&home_dir)
        .arg("--color")
        .arg("never")
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✓ Created config at ~/.eden-skills/skills.toml"),
        "init output should include success symbol with abbreviated path, stdout={stdout}"
    );
    assert!(
        stdout.contains("Next steps:")
            && stdout.contains("eden-skills install <owner/repo>")
            && stdout.contains("eden-skills list")
            && stdout.contains("eden-skills doctor"),
        "init output should include next-steps guidance block, stdout={stdout}"
    );
    assert!(
        !stdout.contains("init: wrote"),
        "legacy init output must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_024_install_per_skill_results() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = setup_local_discovery_repo(
        temp.path(),
        "install-per-skill-repo",
        &[("alpha-skill", "Alpha skill"), ("beta-skill", "Beta skill")],
    );
    let config_path = temp.path().join("skills.toml");

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env_remove("CI")
        .arg("--color")
        .arg("never")
        .args([
            "install",
            &path_as_relative_arg(&repo_dir),
            "--all",
            "--target",
            "claude-code",
            "--target",
            "cursor",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install for per-skill output");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Install"),
        "install output should include Install action prefix, stdout={stdout}"
    );
    assert!(
        stdout.contains("✓ alpha-skill →") && stdout.contains("✓ beta-skill →"),
        "install output should include per-skill per-target result lines, stdout={stdout}"
    );
    assert!(
        stdout.matches("→").count() >= 4,
        "expected 4+ target arrows for 2 skills x 2 targets, stdout={stdout}"
    );
    assert!(
        stdout.contains("(symlink)"),
        "per-target result lines should include install mode, stdout={stdout}"
    );
    assert!(
        stdout.contains("skills installed to") && stdout.contains("conflicts"),
        "final install summary should include skills/agents/conflicts counts, stdout={stdout}"
    );
    assert!(
        !stdout.contains("status=installed"),
        "legacy install summary format must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_025_install_discovery_numbered() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = setup_local_discovery_repo(
        temp.path(),
        "install-discovery-numbered-repo",
        &[
            ("alpha-skill", "Alpha skill"),
            ("beta-skill", "Beta skill"),
            ("gamma-skill", "Gamma skill"),
        ],
    );
    let config_path = temp.path().join("skills.toml");

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env_remove("CI")
        .env("EDEN_SKILLS_TEST_CONFIRM", "y")
        .arg("--color")
        .arg("never")
        .args(["install", &path_as_relative_arg(&repo_dir), "--config"])
        .arg(&config_path)
        .output()
        .expect("run install for discovery output");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Found") && stdout.contains("skills in repository"),
        "discovery summary should include Found action prefix and count, stdout={stdout}"
    );
    assert!(
        stdout.contains("1. alpha-skill")
            && stdout.contains("2. beta-skill")
            && stdout.contains("3. gamma-skill"),
        "discovery output should include numbered skill list, stdout={stdout}"
    );
    assert!(
        stdout.contains("—"),
        "discovery list should include em dash between name and description, stdout={stdout}"
    );
}

fn setup_fixture(
    skill_ids: &[&str],
    target_labels: &[&str],
    verify_checks: &[&str],
    include_license: bool,
) -> Fixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let storage_root = temp.path().join("storage");
    let target_roots = target_labels
        .iter()
        .map(|label| temp.path().join(label))
        .collect::<Vec<_>>();
    for target_root in &target_roots {
        fs::create_dir_all(target_root).expect("create target root");
    }

    let origin_repo = init_git_repo(temp.path(), "origin-repo", include_license);
    let repo_url = as_file_url(&origin_repo);
    let config_path = temp.path().join("skills.toml");
    write_config(
        &config_path,
        &storage_root,
        &repo_url,
        skill_ids,
        &target_roots,
        verify_checks,
    );

    Fixture {
        temp,
        home_dir,
        config_path,
        target_roots,
    }
}

fn write_config(
    config_path: &Path,
    storage_root: &Path,
    repo_url: &str,
    skill_ids: &[&str],
    target_roots: &[PathBuf],
    verify_checks: &[&str],
) {
    let checks = verify_checks
        .iter()
        .map(|check| format!("\"{}\"", toml_escape_str(check)))
        .collect::<Vec<_>>()
        .join(", ");

    let mut config = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n",
        toml_escape_path(storage_root)
    );

    for skill_id in skill_ids {
        config.push_str("\n[[skills]]\n");
        config.push_str(&format!("id = \"{}\"\n\n", toml_escape_str(skill_id)));
        config.push_str("[skills.source]\n");
        config.push_str(&format!("repo = \"{}\"\n", toml_escape_str(repo_url)));
        config.push_str("subpath = \"packages/browser\"\n");
        config.push_str("ref = \"main\"\n\n");
        config.push_str("[skills.install]\n");
        config.push_str("mode = \"symlink\"\n");

        for target_root in target_roots {
            config.push_str("\n[[skills.targets]]\n");
            config.push_str("agent = \"custom\"\n");
            config.push_str(&format!("path = \"{}\"\n", toml_escape_path(target_root)));
        }

        config.push_str("\n[skills.verify]\n");
        config.push_str("enabled = true\n");
        config.push_str(&format!("checks = [{checks}]\n\n"));
        config.push_str("[skills.safety]\n");
        config.push_str("no_exec_metadata_only = false\n");
    }

    fs::write(config_path, config).expect("write test config");
}

fn run_command_with_config(
    fixture: &Fixture,
    color: &str,
    force_tty: bool,
    command_args: &[&str],
    extra_env: &[(&str, &str)],
) -> Output {
    let mut command = eden_command(&fixture.home_dir);
    command
        .current_dir(fixture.temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env_remove("EDEN_SKILLS_TEST_CONFIRM")
        .env_remove("EDEN_SKILLS_TEST_SKILL_INPUT");
    if force_tty {
        command.env("EDEN_SKILLS_FORCE_TTY", "1");
    } else {
        command.env_remove("EDEN_SKILLS_FORCE_TTY");
    }
    for (name, value) in extra_env {
        command.env(name, value);
    }
    command.arg("--color").arg(color);
    command.args(command_args);
    command.arg("--config").arg(&fixture.config_path);
    command.output().expect("run eden-skills command")
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

fn setup_local_discovery_repo(base: &Path, name: &str, skills: &[(&str, &str)]) -> PathBuf {
    let repo_dir = base.join(name);
    for (skill_name, description) in skills {
        let skill_dir = repo_dir.join("skills").join(skill_name);
        fs::create_dir_all(&skill_dir).expect("create local discovery skill dir");
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: {skill_name}\ndescription: {description}\n---\n"),
        )
        .expect("write SKILL.md");
        fs::write(skill_dir.join("README.md"), "demo\n").expect("write README");
    }
    repo_dir
}

fn path_as_relative_arg(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .expect("path should have valid UTF-8 file name");
    format!("./{file_name}")
}

fn init_git_repo(base: &Path, name: &str, include_license: bool) -> PathBuf {
    let repo = base.join(name);
    let browser_dir = repo.join("packages").join("browser");
    fs::create_dir_all(&browser_dir).expect("create browser package");
    fs::write(browser_dir.join("README.md"), "seed\n").expect("write seed readme");
    if include_license {
        fs::write(
            repo.join("LICENSE"),
            "MIT License\n\nPermission is hereby granted, free of charge, to any person obtaining a copy.\n",
        )
        .expect("write license");
    }
    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "test@example.com"]);
    run_git(&repo, &["config", "user.name", "eden-skills-test"]);
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "init"]);
    run_git(&repo, &["branch", "-M", "main"]);
    repo
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

#[cfg(unix)]
fn remove_symlink(path: &Path) -> std::io::Result<()> {
    fs::remove_file(path)
}

#[cfg(windows)]
fn remove_symlink(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => fs::remove_dir(path),
        Err(err) => Err(err),
    }
}
