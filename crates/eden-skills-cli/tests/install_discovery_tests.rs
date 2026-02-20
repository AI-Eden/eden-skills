use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn single_root_skill_installs_without_confirmation_prompt() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("single-repo");
    fs::create_dir_all(&repo_dir).expect("mkdir repo");
    fs::write(
        repo_dir.join("SKILL.md"),
        r#"---
name: root-skill
description: Root skill
---
"#,
    )
    .expect("write skill");
    fs::write(repo_dir.join("README.md"), "demo").expect("write readme");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./single-repo", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let ids = read_skill_ids(&config_path);
    assert_eq!(ids, vec!["root-skill".to_string()]);
}

#[test]
fn skills_directory_discovery_with_all_installs_all_skills() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("multi-skills");
    write_skill(&repo_dir.join("skills/a/SKILL.md"), "skill-a", "A");
    write_skill(&repo_dir.join("skills/b/SKILL.md"), "skill-b", "B");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./multi-skills", "--all", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install --all should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut ids = read_skill_ids(&config_path);
    ids.sort();
    assert_eq!(ids, vec!["skill-a".to_string(), "skill-b".to_string()]);
}

#[test]
fn packages_directory_discovery_with_all_installs_all_skills() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("multi-packages");
    write_skill(&repo_dir.join("packages/x/SKILL.md"), "pkg-x", "X");
    write_skill(&repo_dir.join("packages/y/SKILL.md"), "pkg-y", "Y");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./multi-packages", "--all", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install --all should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut ids = read_skill_ids(&config_path);
    ids.sort();
    assert_eq!(ids, vec!["pkg-x".to_string(), "pkg-y".to_string()]);
}

#[test]
fn missing_skill_markdown_installs_root_with_warning() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("no-skill-md");
    fs::create_dir_all(&repo_dir).expect("mkdir repo");
    fs::write(repo_dir.join("README.md"), "demo").expect("write readme");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./no-skill-md", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("No SKILL.md found; installing directory as-is."),
        "expected warning for missing SKILL.md, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let ids = read_skill_ids(&config_path);
    assert_eq!(ids, vec!["no-skill-md".to_string()]);
}

#[test]
fn list_flag_prints_discovered_skills_without_modifying_config() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("list-repo");
    write_skill(
        &repo_dir.join("skills/a/SKILL.md"),
        "skill-a",
        "A description",
    );
    write_skill(
        &repo_dir.join("skills/b/SKILL.md"),
        "skill-b",
        "B description",
    );

    let config_path = temp.path().join("skills.toml");
    let init_output = eden_command(&home_dir)
        .args(["init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    assert_eq!(
        init_output.status.code(),
        Some(0),
        "init should succeed, stderr={}",
        String::from_utf8_lossy(&init_output.stderr)
    );

    let before = fs::read_to_string(&config_path).expect("read before");
    let list_output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./list-repo", "--list", "--config"])
        .arg(&config_path)
        .output()
        .expect("run list");
    assert_eq!(
        list_output.status.code(),
        Some(0),
        "install --list should succeed, stderr={}",
        String::from_utf8_lossy(&list_output.stderr)
    );
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(stdout.contains("skill-a"), "stdout={stdout}");
    assert!(stdout.contains("skill-b"), "stdout={stdout}");

    let after = fs::read_to_string(&config_path).expect("read after");
    assert_eq!(before, after, "config should not change in --list mode");
}

#[test]
fn skill_flags_install_only_selected_skills() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("selective-repo");
    write_skill(&repo_dir.join("skills/a/SKILL.md"), "skill-a", "A");
    write_skill(&repo_dir.join("skills/b/SKILL.md"), "skill-b", "B");
    write_skill(&repo_dir.join("skills/c/SKILL.md"), "skill-c", "C");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./selective-repo",
            "--skill",
            "skill-a",
            "--skill",
            "skill-c",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install --skill should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut ids = read_skill_ids(&config_path);
    ids.sort();
    assert_eq!(ids, vec!["skill-a".to_string(), "skill-c".to_string()]);
}

#[test]
fn unknown_skill_name_returns_error_with_available_names() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("unknown-skill-repo");
    write_skill(&repo_dir.join("skills/a/SKILL.md"), "skill-a", "A");
    write_skill(&repo_dir.join("skills/b/SKILL.md"), "skill-b", "B");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./unknown-skill-repo",
            "--skill",
            "missing",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(2),
        "unknown skill should return invalid arguments, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("available"), "stderr={stderr}");
    assert!(stderr.contains("skill-a"), "stderr={stderr}");
    assert!(stderr.contains("skill-b"), "stderr={stderr}");
}

#[test]
fn interactive_tty_confirm_yes_installs_all() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("interactive-yes");
    write_skill(&repo_dir.join("skills/a/SKILL.md"), "skill-a", "A");
    write_skill(&repo_dir.join("skills/b/SKILL.md"), "skill-b", "B");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_CONFIRM", "y")
        .args(["install", "./interactive-yes", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "interactive yes should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut ids = read_skill_ids(&config_path);
    ids.sort();
    assert_eq!(ids, vec!["skill-a".to_string(), "skill-b".to_string()]);
}

#[test]
fn interactive_tty_confirm_no_then_selects_named_skills() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("interactive-no");
    write_skill(&repo_dir.join("skills/a/SKILL.md"), "skill-a", "A");
    write_skill(&repo_dir.join("skills/b/SKILL.md"), "skill-b", "B");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_CONFIRM", "n")
        .env("EDEN_SKILLS_TEST_SKILL_INPUT", "skill-b")
        .args(["install", "./interactive-no", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "interactive no + input should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let ids = read_skill_ids(&config_path);
    assert_eq!(ids, vec!["skill-b".to_string()]);
}

#[test]
fn non_tty_defaults_to_install_all_for_multi_skill_repo() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("non-tty-default");
    write_skill(&repo_dir.join("skills/a/SKILL.md"), "skill-a", "A");
    write_skill(&repo_dir.join("skills/b/SKILL.md"), "skill-b", "B");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./non-tty-default", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "non-tty install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut ids = read_skill_ids(&config_path);
    ids.sort();
    assert_eq!(ids, vec!["skill-a".to_string(), "skill-b".to_string()]);
}

#[test]
fn remote_url_with_all_installs_all_discovered_skills() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_git_skill_repo(temp.path(), "remote-multi-all", &["skill-a", "skill-b"]);
    let source = as_file_url(&repo_dir);

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .args(["install", &source, "--all", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "remote install --all should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut ids = read_skill_ids(&config_path);
    ids.sort();
    assert_eq!(ids, vec!["skill-a".to_string(), "skill-b".to_string()]);
}

#[test]
fn remote_url_with_skill_installs_only_selected_skill() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_git_skill_repo(
        temp.path(),
        "remote-multi-select",
        &["skill-a", "skill-b", "skill-c"],
    );
    let source = as_file_url(&repo_dir);

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .args(["install", &source, "--skill", "skill-b", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "remote install --skill should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let ids = read_skill_ids(&config_path);
    assert_eq!(ids, vec!["skill-b".to_string()]);
}

#[test]
fn remote_url_list_does_not_create_config_or_install_targets() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_git_skill_repo(temp.path(), "remote-list-only", &["skill-a", "skill-b"]);
    let source = as_file_url(&repo_dir);

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .args(["install", &source, "--list", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "remote install --list should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("skill-a"), "stdout={stdout}");
    assert!(stdout.contains("skill-b"), "stdout={stdout}");
    assert!(
        !config_path.exists(),
        "--list should not create config file for remote URL source"
    );
    assert!(
        !home_dir.join(".claude/skills").exists(),
        "--list should not install targets"
    );
}

#[test]
fn interactive_summary_truncates_when_more_than_eight_skills() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("truncated-summary");
    for index in 1..=10 {
        write_skill(
            &repo_dir.join(format!("skills/skill-{index}/SKILL.md")),
            &format!("skill-{index}"),
            "demo",
        );
    }

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_CONFIRM", "y")
        .args(["install", "./truncated-summary", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "interactive install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("showing first 8"), "stdout={stdout}");
    assert!(
        stdout.contains("use --list to see all"),
        "stdout should include truncation hint, stdout={stdout}"
    );
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

fn write_skill(skill_md_path: &Path, name: &str, description: &str) {
    fs::create_dir_all(
        skill_md_path
            .parent()
            .expect("skill path should have parent directory"),
    )
    .expect("create skill parent directory");
    fs::write(
        skill_md_path,
        format!("---\nname: {name}\ndescription: {description}\n---\n"),
    )
    .expect("write SKILL.md");
    let skill_dir = skill_md_path
        .parent()
        .expect("skill directory should exist");
    fs::write(skill_dir.join("README.md"), "demo").expect("write skill readme");
}

fn init_git_skill_repo(base: &Path, name: &str, skills: &[&str]) -> std::path::PathBuf {
    let repo = base.join(name);
    for skill in skills {
        write_skill(
            &repo.join(format!("skills/{skill}/SKILL.md")),
            skill,
            &format!("{skill} description"),
        );
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

fn read_skill_ids(config_path: &Path) -> Vec<String> {
    let config_text = fs::read_to_string(config_path).expect("read config");
    let value: toml::Value = toml::from_str(&config_text).expect("valid config toml");
    value
        .get("skills")
        .and_then(|value| value.as_array())
        .map(|skills| {
            skills
                .iter()
                .filter_map(|skill| skill.get("id").and_then(|value| value.as_str()))
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
