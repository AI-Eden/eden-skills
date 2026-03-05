use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use eden_skills_core::lock::{
    lock_path_for_config, read_lock_file, write_lock_file, LockSkillEntry, LockTarget,
};
use tempfile::{tempdir, TempDir};

struct Fixture {
    temp: TempDir,
    home_dir: PathBuf,
    config_path: PathBuf,
    target_roots: Vec<PathBuf>,
    repo_url: String,
}

#[test]
fn tm_p28_012_apply_source_sync_is_styled() {
    let fixture = setup_fixture(&["alpha-skill"], &["agent-a"]);
    let output = run_command_with_config(&fixture, "never", true, &["apply"]);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Syncing"),
        "expected styled Syncing action prefix, stdout={stdout}"
    );
    assert!(
        !stdout.contains("source sync: cloned="),
        "old key=value source sync format must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_013_apply_safety_summary_is_styled() {
    let fixture = setup_fixture(&["alpha-skill"], &["agent-a"]);
    let output = run_command_with_config(&fixture, "never", true, &["apply"]);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Safety"),
        "expected styled Safety action prefix, stdout={stdout}"
    );
    assert!(
        !stdout.contains("safety summary: permissive="),
        "old key=value safety summary must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_014_apply_per_skill_install_lines() {
    let fixture = setup_fixture(&["alpha-skill", "beta-skill"], &["agent-a", "agent-b"]);
    let output = run_command_with_config(&fixture, "never", true, &["apply"]);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Install"),
        "expected Install action section, stdout={stdout}"
    );
    assert!(
        stdout.contains("✓ alpha-skill"),
        "expected grouped install header for alpha-skill, stdout={stdout}"
    );
    assert!(
        stdout.contains("✓ beta-skill"),
        "expected grouped install header for beta-skill, stdout={stdout}"
    );
    assert!(
        stdout.matches("├─").count() >= 2 && stdout.matches("└─").count() >= 2,
        "expected tree-style per-target lines for both skills, stdout={stdout}"
    );
    assert!(
        stdout.contains("(symlink)"),
        "expected install mode marker on per-target lines, stdout={stdout}"
    );
    assert!(
        !stdout.contains("→"),
        "legacy arrow line format should be removed from apply install output, stdout={stdout}"
    );
}

#[test]
fn tm_p28_015_apply_summary_is_styled() {
    let fixture = setup_fixture(&["alpha-skill"], &["agent-a"]);
    let output = run_command_with_config(&fixture, "never", true, &["apply"]);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Summary"),
        "expected Summary action prefix in apply output, stdout={stdout}"
    );
    assert!(
        stdout.contains("created")
            && stdout.contains("updated")
            && stdout.contains("noop")
            && stdout.contains("conflicts"),
        "expected styled aggregate summary counts, stdout={stdout}"
    );
    assert!(
        !stdout.contains("apply summary: create="),
        "old apply summary key=value line must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_016_apply_verification_is_styled() {
    let fixture = setup_fixture(&["alpha-skill"], &["agent-a"]);
    let output = run_command_with_config(&fixture, "never", true, &["apply"]);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✓ Verification passed"),
        "expected styled verification success line, stdout={stdout}"
    );
    assert!(
        !stdout.contains("apply verification: ok"),
        "old apply verification line must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_017_repair_output_matches_apply_format() {
    let fixture = setup_fixture(&["alpha-skill"], &["agent-a"]);
    let first_apply = run_command_with_config(&fixture, "never", true, &["apply"]);
    assert_success(&first_apply);

    let target = fixture.target_roots[0].join("alpha-skill");
    remove_symlink(&target).expect("remove existing symlink");
    let broken_target = fixture.temp.path().join("broken-target");
    create_symlink(&broken_target, &target).expect("write broken symlink");

    let repair = run_command_with_config(&fixture, "never", true, &["repair"]);
    assert_success(&repair);

    let stdout = String::from_utf8_lossy(&repair.stdout);
    assert!(
        stdout.contains("Syncing"),
        "repair output should include styled Syncing line, stdout={stdout}"
    );
    assert!(
        stdout.contains("Safety"),
        "repair output should include styled Safety line, stdout={stdout}"
    );
    assert!(
        stdout.contains("Install"),
        "repair output should include per-target Install lines, stdout={stdout}"
    );
    assert!(
        stdout.contains("Verification passed"),
        "repair output should include styled verification line, stdout={stdout}"
    );
    assert!(
        !stdout.contains("repair summary:"),
        "old repair summary format must be removed, stdout={stdout}"
    );
    assert!(
        !stdout.contains("repair verification: ok"),
        "old repair verification format must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_021_plan_header_and_colored_actions() {
    let fixture = setup_fixture(
        &["create-skill", "noop-skill", "conflict-skill"],
        &["agent-plan"],
    );
    let apply_output = run_command_with_config(&fixture, "never", false, &["apply"]);
    assert_success(&apply_output);

    let create_target = fixture.target_roots[0].join("create-skill");
    remove_symlink(&create_target).expect("remove create target for create action");

    let conflict_target = fixture.target_roots[0].join("conflict-skill");
    remove_symlink(&conflict_target).expect("remove conflict target symlink");
    fs::create_dir_all(&conflict_target).expect("create conflicting directory target");

    let lock_path = lock_path_for_config(&fixture.config_path);
    let mut lock = read_lock_file(&lock_path)
        .expect("read lock file")
        .expect("lock should exist after apply");
    lock.skills.push(LockSkillEntry {
        id: "removed-skill".to_string(),
        source_repo: fixture.repo_url.clone(),
        source_subpath: "packages/browser".to_string(),
        source_ref: "main".to_string(),
        resolved_commit: "deadbeef".to_string(),
        resolved_version: None,
        install_mode: "symlink".to_string(),
        installed_at: "2026-03-05T00:00:00Z".to_string(),
        targets: vec![LockTarget {
            agent: "custom".to_string(),
            path: fixture.target_roots[0]
                .join("removed-skill")
                .display()
                .to_string(),
            environment: "local".to_string(),
        }],
    });
    write_lock_file(&lock_path, &lock).expect("write lock with orphan entry");

    let plan_output = run_command_with_config(&fixture, "always", true, &["plan"]);
    assert_success(&plan_output);

    let stdout = String::from_utf8_lossy(&plan_output.stdout);
    assert!(
        stdout.contains("Plan") && stdout.contains("actions"),
        "expected plan header with action count, stdout={stdout}"
    );
    assert!(
        stdout.contains("create-skill")
            && stdout.contains("noop-skill")
            && stdout.contains("conflict-skill")
            && stdout.contains("removed-skill"),
        "expected create/noop/conflict/remove action rows, stdout={stdout}"
    );
    assert!(
        stdout.contains("→"),
        "expected unicode arrow in plan paths, stdout={stdout}"
    );
    assert!(
        !stdout.contains("->"),
        "old ASCII arrow should be removed from plan output, stdout={stdout}"
    );
    assert!(
        has_ansi_codes(&stdout),
        "expected ANSI colors for plan action labels, stdout={stdout}"
    );
    assert!(
        stdout.contains("\u{1b}[32m"),
        "expected green create action styling, stdout={stdout}"
    );
    assert!(
        stdout.contains("\u{1b}[33m"),
        "expected yellow conflict action styling, stdout={stdout}"
    );
    assert!(
        stdout.contains("\u{1b}[31m"),
        "expected red remove action styling, stdout={stdout}"
    );
    assert!(
        stdout.contains("\u{1b}[2m") || stdout.contains("\u{1b}[90m"),
        "expected dim noop action styling, stdout={stdout}"
    );
}

#[test]
fn tm_p28_022_plan_empty_state() {
    let fixture = setup_fixture(&["alpha-skill"], &["agent-a"]);
    let apply_output = run_command_with_config(&fixture, "never", true, &["apply"]);
    assert_success(&apply_output);

    let plan_output = run_command_with_config(&fixture, "never", true, &["plan"]);
    assert_success(&plan_output);

    let stdout = String::from_utf8_lossy(&plan_output.stdout);
    assert!(
        stdout.contains("Plan"),
        "expected plan header in empty state, stdout={stdout}"
    );
    assert!(
        stdout.contains("✓ 0 actions (up to date)"),
        "expected styled up-to-date plan empty state, stdout={stdout}"
    );
}

#[test]
fn tm_p28_026_error_hint_uses_arrow() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");

    let output = eden_command(&home_dir)
        .arg("--color")
        .arg("always")
        .arg("list")
        .output()
        .expect("run list with missing default config");

    assert_eq!(
        output.status.code(),
        Some(1),
        "missing config should exit with code 1, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("→"),
        "expected arrow-prefixed hint, stderr={stderr}"
    );
    assert!(
        !stderr.contains("hint:"),
        "legacy `hint:` prefix must be removed, stderr={stderr}"
    );
    assert!(
        has_ansi_codes(&stderr),
        "expected colorized error output when --color always is set, stderr={stderr}"
    );
}

#[test]
fn tm_p28_027_error_path_is_abbreviated() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");

    let output = eden_command(&home_dir)
        .arg("--color")
        .arg("never")
        .arg("list")
        .output()
        .expect("run list with missing default config");

    assert_eq!(
        output.status.code(),
        Some(1),
        "missing config should exit with code 1, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("config file not found: ~/.eden-skills/skills.toml"),
        "expected HOME-abbreviated config path in error output, stderr={stderr}"
    );
    assert!(
        !stderr.contains(&home_dir.display().to_string()),
        "absolute HOME path should not appear in human error output, stderr={stderr}"
    );
}

#[test]
fn tm_p28_028_warning_format_is_styled() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let storage_root = temp.path().join("storage");
    let config_path = temp.path().join("skills.toml");

    fs::write(
        &config_path,
        format!(
            r#"version = 1

[storage]
root = "{storage_root}"

[mystery]
enabled = true
"#,
            storage_root = toml_escape_path(&storage_root),
        ),
    )
    .expect("write warning fixture config");

    let no_color = eden_command(&home_dir)
        .arg("--color")
        .arg("never")
        .args(["plan", "--config"])
        .arg(&config_path)
        .output()
        .expect("run plan --color never");
    assert_success(&no_color);
    let stderr_no_color = String::from_utf8_lossy(&no_color.stderr);
    assert!(
        stderr_no_color.contains("  warning:"),
        "warning lines must use 2-space indent, stderr={stderr_no_color}"
    );
    assert!(
        !has_ansi_codes(&stderr_no_color),
        "warnings should remain plain in --color never mode, stderr={stderr_no_color}"
    );

    let color_always = eden_command(&home_dir)
        .arg("--color")
        .arg("always")
        .args(["plan", "--config"])
        .arg(&config_path)
        .output()
        .expect("run plan --color always");
    assert_success(&color_always);
    let stderr_color = String::from_utf8_lossy(&color_always.stderr);
    assert!(
        stderr_color.contains("warning:"),
        "warning prefix should remain visible under colorized output, stderr={stderr_color}"
    );
    assert!(
        has_ansi_codes(&stderr_color),
        "warning prefix should be colorized when --color always is used, stderr={stderr_color}"
    );
}

fn setup_fixture(skill_ids: &[&str], target_labels: &[&str]) -> Fixture {
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

    let origin_repo = init_git_repo(
        temp.path(),
        "origin-repo",
        &[("packages/browser/README.md", "seed\n")],
    );
    let repo_url = as_file_url(&origin_repo);
    let config_path = temp.path().join("skills.toml");
    write_config(
        &config_path,
        &storage_root,
        &repo_url,
        skill_ids,
        &target_roots,
    );

    Fixture {
        temp,
        home_dir,
        config_path,
        target_roots,
        repo_url,
    }
}

fn write_config(
    config_path: &Path,
    storage_root: &Path,
    repo_url: &str,
    skill_ids: &[&str],
    target_roots: &[PathBuf],
) {
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
        config.push_str("checks = [\"path-exists\", \"target-resolves\", \"is-symlink\"]\n\n");
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
) -> Output {
    let mut command = eden_command(&fixture.home_dir);
    command
        .current_dir(fixture.temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI");
    if force_tty {
        command.env("EDEN_SKILLS_FORCE_TTY", "1");
    } else {
        command.env_remove("EDEN_SKILLS_FORCE_TTY");
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

fn init_git_repo(base: &Path, name: &str, files: &[(&str, &str)]) -> PathBuf {
    let repo = base.join(name);
    for (rel, content) in files {
        let path = repo.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(path, content).expect("write repo file");
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

fn has_ansi_codes(text: &str) -> bool {
    text.as_bytes().windows(2).any(|window| window == b"\x1b[")
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

#[cfg(unix)]
fn create_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(source, target)
}
