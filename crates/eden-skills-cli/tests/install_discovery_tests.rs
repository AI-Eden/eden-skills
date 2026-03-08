mod common;

use std::fs;
use std::path::Path;

use serde_json::Value;
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
    let output = common::eden_command(&home_dir)
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
    let output = common::eden_command(&home_dir)
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
    let output = common::eden_command(&home_dir)
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
    let output = common::eden_command(&home_dir)
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
fn single_discovered_skill_with_unmatched_skill_flag_returns_error() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("single-skill-mismatch");
    write_skill(&repo_dir.join("SKILL.md"), "single-skill", "demo");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./single-skill-mismatch",
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
        "unmatched --skill should return invalid arguments, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown skill name"), "stderr={stderr}");
    assert!(stderr.contains("single-skill"), "stderr={stderr}");
}

#[test]
fn missing_skill_markdown_with_skill_flag_returns_error_instead_of_root_fallback() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("no-skill-md-select");
    fs::create_dir_all(&repo_dir).expect("mkdir repo");
    fs::write(repo_dir.join("README.md"), "demo").expect("write readme");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./no-skill-md-select",
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
        "missing discovery + --skill should fail, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_no_skills_persisted(&config_path);
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
    let init_output = common::eden_command(&home_dir)
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
    let list_output = common::eden_command(&home_dir)
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
fn tm_p29_015_install_list_shows_card_style_numbered_list() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("card-list-repo");
    write_skill(
        &repo_dir.join("skills/alpha/SKILL.md"),
        "alpha-skill",
        "Alpha details",
    );
    write_skill(
        &repo_dir.join("skills/beta/SKILL.md"),
        "beta-skill",
        "Beta details",
    );

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "--color",
            "never",
            "install",
            "./card-list-repo",
            "--list",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install --list");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install --list should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Found") && stdout.contains("skills in repository"),
        "card preview should include discovery header, stdout={stdout}"
    );
    assert!(
        stdout.contains("    1. alpha-skill") && stdout.contains("    2. beta-skill"),
        "card preview should render numbered list lines, stdout={stdout}"
    );
    assert!(
        !stdout.contains("| Name") && !stdout.contains("+---"),
        "install --list should not use table rendering, stdout={stdout}"
    );
}

#[test]
fn tm_p29_017_discovery_description_uses_indented_followup_line() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("description-indent-repo");
    write_skill(
        &repo_dir.join("skills/alpha/SKILL.md"),
        "alpha-skill",
        "Alpha details should be shown on a separate line.",
    );

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "--color",
            "never",
            "install",
            "./description-indent-repo",
            "--list",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install --list");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install --list should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(
            "    1. alpha-skill\n       Alpha details should be shown on a separate line."
        ),
        "description should be rendered on an indented line below skill name, stdout={stdout}"
    );
}

#[test]
fn tm_p29_018_discovery_skill_without_description_renders_name_only_line() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("name-only-repo");
    write_skill_without_description(&repo_dir.join("skills/plain/SKILL.md"), "plain-skill");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "--color",
            "never",
            "install",
            "./name-only-repo",
            "--list",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install --list");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install --list should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("    1. plain-skill"),
        "name-only skill should still render numbered name line, stdout={stdout}"
    );
    assert!(
        !stdout.contains("    1. plain-skill\n       "),
        "name-only skill must not render a followup description line, stdout={stdout}"
    );
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
    let output = common::eden_command(&home_dir)
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
    let output = common::eden_command(&home_dir)
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
fn interactive_tty_test_indices_install_all() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("interactive-yes");
    write_skill(&repo_dir.join("skills/a/SKILL.md"), "skill-a", "A");
    write_skill(&repo_dir.join("skills/b/SKILL.md"), "skill-b", "B");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_SKILL_INPUT", "0,1")
        .args(["install", "./interactive-yes", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "interactive index selection should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut ids = read_skill_ids(&config_path);
    ids.sort();
    assert_eq!(ids, vec!["skill-a".to_string(), "skill-b".to_string()]);
}

#[test]
fn interactive_tty_test_indices_select_named_skills() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("interactive-no");
    write_skill(&repo_dir.join("skills/a/SKILL.md"), "skill-a", "A");
    write_skill(&repo_dir.join("skills/b/SKILL.md"), "skill-b", "B");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_SKILL_INPUT", "1")
        .args(["install", "./interactive-no", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "interactive index selection should succeed, stderr={}",
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
    let output = common::eden_command(&home_dir)
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
    let source = common::path_to_file_url(&repo_dir);

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
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
    let source = common::path_to_file_url(&repo_dir);

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
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
    let source = common::path_to_file_url(&repo_dir);

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
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
fn agent_convention_skill_directory_supports_skill_flag_selection() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("agent-convention-repo");
    write_skill(
        &repo_dir.join(".claude/skills/pdf/SKILL.md"),
        "pdf",
        "PDF helpers",
    );
    write_skill(
        &repo_dir.join(".claude/skills/xlsx/SKILL.md"),
        "xlsx",
        "Spreadsheet helpers",
    );

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./agent-convention-repo",
            "--skill",
            "pdf",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install --skill should succeed for agent convention directories, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let ids = read_skill_ids(&config_path);
    assert_eq!(ids, vec!["pdf".to_string()]);
}

#[test]
fn recursive_fallback_discovery_supports_skill_flag_selection() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("recursive-fallback-repo");
    write_skill(
        &repo_dir.join("vendor/tools/pdf/SKILL.md"),
        "pdf",
        "PDF helpers",
    );
    write_skill(
        &repo_dir.join("vendor/tools/xlsx/SKILL.md"),
        "xlsx",
        "XLSX helpers",
    );

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "install",
            "./recursive-fallback-repo",
            "--skill",
            "pdf",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install --skill should succeed with recursive fallback discovery, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let ids = read_skill_ids(&config_path);
    assert_eq!(ids, vec!["pdf".to_string()]);
}

#[test]
fn remote_url_missing_skill_markdown_with_skill_flag_returns_error() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_git_repo_without_skill(temp.path(), "remote-no-skill-select");
    let source = common::path_to_file_url(&repo_dir);

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .args(["install", &source, "--skill", "missing", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(2),
        "remote missing discovery + --skill should fail, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_no_skills_persisted(&config_path);
}

#[test]
fn tm_p29_020_source_sync_shows_step_style_progress_in_tty() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_git_skill_repo(temp.path(), "remote-progress-tty", &["skill-a", "skill-b"]);
    let source = common::path_to_file_url(&repo_dir);
    let config_path = temp.path().join("skills.toml");

    let output = common::eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .args([
            "--color",
            "never",
            "install",
            &source,
            "--all",
            "--target",
            "claude-code",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let combined = combined_output_text(&output);
    assert!(
        combined.contains("Syncing"),
        "sync output should include Syncing progress prefix, output={combined}"
    );
    assert!(
        combined.contains("[1/2]") || combined.contains("[2/2]"),
        "TTY sync output should include step-style [pos/len] markers, output={combined}"
    );
}

#[test]
fn tm_p29_021_source_sync_prints_summary_line_after_completion() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_git_skill_repo(temp.path(), "remote-sync-summary-tty", &["skill-a"]);
    let source = common::path_to_file_url(&repo_dir);
    let config_path = temp.path().join("skills.toml");

    let output = common::eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .args([
            "--color",
            "never",
            "install",
            &source,
            "--all",
            "--target",
            "claude-code",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Syncing") && stdout.contains("synced") && stdout.contains("failed"),
        "sync output should include completion summary line, stdout={stdout}"
    );
    assert!(
        !stdout.contains("source sync:"),
        "legacy source sync key-value line should be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p29_022_non_tty_source_sync_skips_progress_bar_and_keeps_summary() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = init_git_skill_repo(temp.path(), "remote-sync-summary-non-tty", &["skill-a"]);
    let source = common::path_to_file_url(&repo_dir);
    let config_path = temp.path().join("skills.toml");

    let output = common::eden_command(&home_dir)
        .env_remove("EDEN_SKILLS_FORCE_TTY")
        .args([
            "--color",
            "never",
            "install",
            &source,
            "--all",
            "--target",
            "claude-code",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = combined_output_text(&output);
    assert!(
        stdout.contains("Syncing") && stdout.contains("synced") && stdout.contains("failed"),
        "non-TTY sync should still print compact summary line, stdout={stdout}"
    );
    assert!(
        !combined.contains("[1/1]"),
        "non-TTY sync should not render step progress markers, output={combined}"
    );
}

#[test]
fn tm_p29_023_install_results_use_tree_display_with_connectors() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("tree-install-repo");
    write_skill(
        &repo_dir.join("skills/alpha/SKILL.md"),
        "alpha-skill",
        "Alpha details",
    );
    write_skill(
        &repo_dir.join("skills/beta/SKILL.md"),
        "beta-skill",
        "Beta details",
    );

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "--color",
            "never",
            "install",
            "./tree-install-repo",
            "--all",
            "--target",
            "claude-code",
            "--target",
            "cursor",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Install"),
        "tree output should include Install action header, stdout={stdout}"
    );
    assert!(
        stdout.contains("├─") && stdout.contains("└─"),
        "tree output should include branch connectors, stdout={stdout}"
    );
    assert!(
        stdout.contains("(symlink)"),
        "tree output should include install mode labels, stdout={stdout}"
    );
    assert!(
        !stdout.contains("~>"),
        "legacy flat arrow output must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p29_024_tree_groups_skill_name_once_per_skill_group() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("tree-group-repo");
    write_skill(
        &repo_dir.join("skills/alpha/SKILL.md"),
        "alpha-skill",
        "Alpha details",
    );
    write_skill(
        &repo_dir.join("skills/beta/SKILL.md"),
        "beta-skill",
        "Beta details",
    );

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "--color",
            "never",
            "install",
            "./tree-group-repo",
            "--all",
            "--target",
            "claude-code",
            "--target",
            "cursor",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.matches("✓ alpha-skill").count(),
        1,
        "alpha-skill should appear once as grouped skill header, stdout={stdout}"
    );
    assert_eq!(
        stdout.matches("✓ beta-skill").count(),
        1,
        "beta-skill should appear once as grouped skill header, stdout={stdout}"
    );
}

#[test]
fn tm_p29_027_install_list_json_contract_returns_discovered_skill_array() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("list-json-repo");
    write_skill(
        &repo_dir.join("skills/alpha/SKILL.md"),
        "alpha-skill",
        "Alpha details",
    );
    write_skill(
        &repo_dir.join("skills/beta/SKILL.md"),
        "beta-skill",
        "Beta details",
    );

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "--color",
            "never",
            "install",
            "./list-json-repo",
            "--list",
            "--json",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install --list --json");
    assert_eq!(
        output.status.code(),
        Some(0),
        "install --list --json should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|err| panic!("stdout must be json: {err}\nstdout={stdout}"));
    let items = payload
        .as_array()
        .expect("install --list --json should return a JSON array");
    assert_eq!(
        items.len(),
        2,
        "expected two discovered skills, json={payload}"
    );

    let mut names = items
        .iter()
        .map(|item| {
            assert!(
                item.get("description").and_then(Value::as_str).is_some(),
                "each discovered skill must include description string, json={payload}"
            );
            assert!(
                item.get("subpath").and_then(Value::as_str).is_some(),
                "each discovered skill must include subpath string, json={payload}"
            );
            item.get("name")
                .and_then(Value::as_str)
                .expect("each discovered skill must include name string")
                .to_string()
        })
        .collect::<Vec<_>>();
    names.sort();
    assert_eq!(
        names,
        vec!["alpha-skill".to_string(), "beta-skill".to_string()],
        "json names should match discovered skills, json={payload}"
    );
    assert!(
        !config_path.exists(),
        "--list --json should not create config file"
    );
}

#[test]
fn interactive_confirm_interrupt_cancels_without_error_output() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("interactive-interrupt-repo");
    write_skill(&repo_dir.join("skills/a/SKILL.md"), "skill-a", "A");
    write_skill(&repo_dir.join("skills/b/SKILL.md"), "skill-b", "B");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_SKILL_INPUT", "interrupt")
        .args(["install", "./interactive-interrupt-repo", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install with interrupted prompt");
    assert_eq!(
        output.status.code(),
        Some(0),
        "interrupted prompt should cancel install without error, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("◆  Install canceled"),
        "interrupted prompt should emit cancellation line, stdout={stdout}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).trim().is_empty(),
        "interrupted prompt should not emit runtime error to stderr, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ids = read_skill_ids(&config_path);
    assert!(
        ids.is_empty(),
        "interrupted prompt should not persist selected skills"
    );
}

#[test]
fn dry_run_multi_skill_preview_defaults_to_eight_skill_rows() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("dry-run-many-skills");
    for index in 1..=10 {
        write_skill(
            &repo_dir.join(format!("skills/skill-{index}/SKILL.md")),
            &format!("skill-{index}"),
            &format!("Skill {index} description"),
        );
    }

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "--color",
            "never",
            "install",
            "./dry-run-many-skills",
            "--all",
            "--dry-run",
            "--target",
            "claude-code",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run dry-run install");
    assert_eq!(
        output.status.code(),
        Some(0),
        "dry-run install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Skill / Version / Source"),
        "dry-run should include skill preview title, stdout={stdout}"
    );
    assert!(
        stdout.contains("Install Targets"),
        "dry-run should include targets preview title, stdout={stdout}"
    );
    assert!(
        stdout.contains("    "),
        "dry-run tables should be indented by 4 spaces, stdout={stdout}"
    );

    let skill_table = extract_titled_table_block(&stdout, "Skill / Version / Source");
    assert!(
        skill_table.contains("| #")
            && skill_table.contains("| Skill")
            && skill_table.contains("| Version")
            && skill_table.contains("| Source"),
        "skill preview should render table headers, table={skill_table}"
    );
    assert!(
        skill_table.contains("| 8"),
        "default dry-run skill table should include row #8, table={skill_table}"
    );
    assert!(
        !skill_table.contains("| 9"),
        "default dry-run skill table should truncate rows beyond 8, table={skill_table}"
    );
    assert!(
        stdout.contains("... and 2 more (use --dry-run --list to show all)"),
        "dry-run output should include truncation footer, stdout={stdout}"
    );

    let target_table = extract_titled_table_block(&stdout, "Install Targets");
    assert!(
        target_table.contains("| Agent")
            && target_table.contains("| Path")
            && target_table.contains("| Mode"),
        "target preview should keep Agent/Path/Mode columns, table={target_table}"
    );
    assert!(
        !target_table.contains("| Skill"),
        "target preview table should not include Skill column, table={target_table}"
    );
}

#[test]
fn dry_run_multi_skill_with_list_shows_all_skill_rows() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("dry-run-many-skills-list");
    for index in 1..=10 {
        write_skill(
            &repo_dir.join(format!("skills/skill-{index}/SKILL.md")),
            &format!("skill-{index}"),
            &format!("Skill {index} description"),
        );
    }

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .args([
            "--color",
            "never",
            "install",
            "./dry-run-many-skills-list",
            "--all",
            "--dry-run",
            "--list",
            "--target",
            "claude-code",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run dry-run install --list");
    assert_eq!(
        output.status.code(),
        Some(0),
        "dry-run install --list should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let skill_table = extract_titled_table_block(&stdout, "Skill / Version / Source");
    assert!(
        skill_table.contains("| 10"),
        "dry-run --list skill table should include all rows, table={skill_table}"
    );
    assert!(
        !stdout.contains("use --dry-run --list to show all"),
        "dry-run --list should not emit truncation footer, stdout={stdout}"
    );
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

fn write_skill_without_description(skill_md_path: &Path, name: &str) {
    fs::create_dir_all(
        skill_md_path
            .parent()
            .expect("skill path should have parent directory"),
    )
    .expect("create skill parent directory");
    fs::write(skill_md_path, format!("---\nname: {name}\n---\n")).expect("write SKILL.md");
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
    common::run_git_cmd(&repo, &["init"]);
    common::run_git_cmd(&repo, &["config", "user.email", common::TEST_GIT_EMAIL]);
    common::run_git_cmd(&repo, &["config", "user.name", common::TEST_GIT_NAME]);
    common::run_git_cmd(&repo, &["add", "."]);
    common::run_git_cmd(&repo, &["commit", "-m", "init"]);
    common::run_git_cmd(&repo, &["branch", "-M", "main"]);
    repo
}

fn init_git_repo_without_skill(base: &Path, name: &str) -> std::path::PathBuf {
    let repo = base.join(name);
    fs::create_dir_all(&repo).expect("mkdir repo");
    fs::write(repo.join("README.md"), "demo").expect("write readme");
    common::run_git_cmd(&repo, &["init"]);
    common::run_git_cmd(&repo, &["config", "user.email", common::TEST_GIT_EMAIL]);
    common::run_git_cmd(&repo, &["config", "user.name", common::TEST_GIT_NAME]);
    common::run_git_cmd(&repo, &["add", "."]);
    common::run_git_cmd(&repo, &["commit", "-m", "init"]);
    common::run_git_cmd(&repo, &["branch", "-M", "main"]);
    repo
}

fn combined_output_text(output: &std::process::Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn extract_titled_table_block(stdout: &str, title: &str) -> String {
    let mut lines = Vec::new();
    let mut started = false;
    let title_line = format!("  {title}");
    for line in stdout.lines() {
        if !started {
            if line == title_line {
                started = true;
            }
            continue;
        }
        if line.is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }
        if line.starts_with("  ") && !line.starts_with("    ") {
            break;
        }
        lines.push(line.to_string());
    }
    lines.join("\n")
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

fn assert_no_skills_persisted(config_path: &Path) {
    if !config_path.exists() {
        return;
    }
    let ids = read_skill_ids(config_path);
    assert!(
        ids.is_empty(),
        "failed selection should not persist skill entries, config={}",
        config_path.display()
    );
}
