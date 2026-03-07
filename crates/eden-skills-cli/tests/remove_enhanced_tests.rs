use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use eden_skills_core::config::{load_from_file, LoadOptions};
use eden_skills_core::lock::{
    build_lock_from_config, lock_path_for_config, read_lock_file, write_lock_file,
};
use tempfile::tempdir;
use toml::Value;

#[test]
fn batch_remove_multiple_skills_updates_config_and_lock() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["a", "b", "c"]);
    write_lock_snapshot(&config_path);

    let output = eden_command(&home_dir)
        .args(["remove", "a", "c", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run batch remove");
    assert_success(&output);

    let remaining = read_skill_ids(&config_path);
    assert_eq!(remaining, vec!["b".to_string()]);

    let lock = read_lock_file(&lock_path_for_config(&config_path))
        .expect("read lock")
        .expect("lock exists");
    let lock_ids = lock
        .skills
        .into_iter()
        .map(|entry| entry.id)
        .collect::<Vec<_>>();
    assert_eq!(lock_ids, vec!["b".to_string()]);
}

#[test]
fn batch_remove_unknown_id_fails_atomically_without_partial_removal() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["a", "b"]);
    write_lock_snapshot(&config_path);

    let output = eden_command(&home_dir)
        .args(["remove", "a", "nonexistent", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run batch remove with unknown id");
    assert_eq!(
        output.status.code(),
        Some(2),
        "unknown id should return invalid arguments, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown skill(s): 'nonexistent'"),
        "stderr={stderr}"
    );
    assert!(stderr.contains("Available skills: a, b"), "stderr={stderr}");

    let remaining = read_skill_ids(&config_path);
    assert_eq!(remaining, vec!["a".to_string(), "b".to_string()]);

    let lock = read_lock_file(&lock_path_for_config(&config_path))
        .expect("read lock")
        .expect("lock exists");
    let lock_ids = lock
        .skills
        .into_iter()
        .map(|entry| entry.id)
        .collect::<Vec<_>>();
    assert_eq!(lock_ids, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn remove_without_args_on_tty_enters_interactive_selection_mode() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["a", "b", "c"]);

    let output = eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "0,2")
        .env("EDEN_SKILLS_TEST_CONFIRM", "y")
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run interactive remove");
    assert_success(&output);

    let remaining = read_skill_ids(&config_path);
    assert_eq!(remaining, vec!["b".to_string()]);
}

#[test]
fn remove_without_args_selects_all_skills_from_zero_based_indices() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["a", "b", "c"]);

    let output = eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "0,1,2")
        .env("EDEN_SKILLS_TEST_CONFIRM", "y")
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run interactive remove with all indices");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first = stdout
        .find("  Remove  ✓ a")
        .expect("summary should include first skill");
    let second = stdout
        .find("          ✓ b")
        .expect("summary should include second skill");
    let third = stdout
        .find("          ✓ c")
        .expect("summary should include third skill");
    assert!(
        first < second && second < third,
        "selection order should preserve config order in summary, stdout={stdout}"
    );

    let remaining = read_skill_ids(&config_path);
    assert!(
        remaining.is_empty(),
        "index selection should remove all configured skills"
    );
}

#[test]
fn remove_without_args_rejects_star_as_special_remove_syntax() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["a", "b", "c"]);

    let output = eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "*")
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run interactive remove with star selection");
    assert_eq!(
        output.status.code(),
        Some(2),
        "mixed wildcard input should fail, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid interactive selection index"),
        "stderr={stderr}"
    );

    let remaining = read_skill_ids(&config_path);
    assert_eq!(
        remaining,
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        "star input should leave config unchanged"
    );
}

#[test]
fn remove_without_args_empty_confirmation_uses_default_no_and_cancels() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["a", "b", "c"]);

    let output = eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "0,2")
        .env("EDEN_SKILLS_TEST_CONFIRM", "")
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run interactive remove with empty confirmation");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("  · Remove cancelled"),
        "empty confirmation should use default no and cancel, stdout={stdout}"
    );

    let remaining = read_skill_ids(&config_path);
    assert_eq!(
        remaining,
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        "empty confirmation should keep config unchanged"
    );
}

#[test]
fn remove_without_args_on_non_tty_fails_with_usage_hint() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["a"]);

    let output = eden_command(&home_dir)
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run remove without ids in non-tty");
    assert_eq!(
        output.status.code(),
        Some(2),
        "non-tty remove without ids should fail, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no skill IDs specified"), "stderr={stderr}");
    assert!(
        stderr.contains("Usage: eden-skills remove <SKILL_ID>..."),
        "stderr={stderr}"
    );
}

#[test]
fn remove_yes_flag_skips_confirmation_prompt() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["browser-tool"]);

    let output = eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_CONFIRM", "n")
        .args([
            "remove",
            "browser-tool",
            "-y",
            "--color",
            "never",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run remove --yes");
    assert_success(&output);

    let remaining = read_skill_ids(&config_path);
    assert!(
        remaining.is_empty(),
        "all configured skills should be removed"
    );
}

#[test]
fn remove_index_selection_yes_flag_skips_confirmation_prompt_and_removes_all() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["a", "b", "c"]);

    let output = eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "0,1,2")
        .args(["remove", "-y", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run remove -y with index selection");
    assert_success(&output);

    let remaining = read_skill_ids(&config_path);
    assert!(
        remaining.is_empty(),
        "index selection with -y should remove all configured skills"
    );
}

#[test]
fn remove_confirm_interrupt_is_handled_as_graceful_cancellation() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["interrupt-me"]);

    let output = eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_CONFIRM", "interrupt")
        .args(["remove", "interrupt-me", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run remove with interrupted confirmation");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("◆  Remove canceled"),
        "interrupted confirmation should emit cancellation line, stdout={stdout}"
    );
    let remaining = read_skill_ids(&config_path);
    assert_eq!(
        remaining,
        vec!["interrupt-me".to_string()],
        "interrupted confirmation should keep config unchanged"
    );
}

#[test]
fn remove_without_args_declined_confirmation_cancels_removal() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["a", "b", "c"]);

    let output = eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "0,2")
        .env("EDEN_SKILLS_TEST_CONFIRM", "n")
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run interactive remove with declined confirmation");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("  · Remove cancelled"),
        "declined wildcard confirmation should emit cancellation line, stdout={stdout}"
    );

    let remaining = read_skill_ids(&config_path);
    assert_eq!(
        remaining,
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        "declined confirmation should keep config unchanged"
    );
}

#[test]
fn remove_selection_interrupt_is_handled_as_graceful_cancellation() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["a", "b"]);

    let output = eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "interrupt")
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run interactive remove with interrupted selection");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("◆  Remove canceled"),
        "interrupted selection should emit cancellation line, stdout={stdout}"
    );
    let remaining = read_skill_ids(&config_path);
    assert_eq!(
        remaining,
        vec!["a".to_string(), "b".to_string()],
        "interrupted selection should keep config unchanged"
    );
}

#[test]
fn install_yes_flag_skips_prompts_for_multi_skill_repo() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("yes-install-repo");
    write_skill(&repo_dir.join("skills/a/SKILL.md"), "skill-a", "A");
    write_skill(&repo_dir.join("skills/b/SKILL.md"), "skill-b", "B");
    let config_path = temp.path().join("skills.toml");

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_CONFIRM", "n")
        .args(["install", "./yes-install-repo", "-y", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install -y on multi-skill repo");
    assert_success(&output);

    let mut installed = read_skill_ids(&config_path);
    installed.sort();
    assert_eq!(
        installed,
        vec!["skill-a".to_string(), "skill-b".to_string()]
    );
}

#[test]
fn remove_without_args_on_empty_config_reports_nothing_to_remove() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    write_empty_config(&config_path, &temp.path().join("storage"));

    let output = eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run remove with empty config");
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Skills   0 configured"), "stdout={stdout}");
    assert!(stdout.contains("Nothing to remove."), "stdout={stdout}");
}

#[test]
fn batch_remove_json_output_contains_removed_array() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(&config_path, &storage_root, &target_root, &["a", "b", "c"]);

    let output = eden_command(&home_dir)
        .args(["remove", "a", "b", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run batch remove --json");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(&stdout).expect("remove --json should be valid JSON");
    assert_eq!(
        payload.get("action").and_then(Value::as_str),
        Some("remove")
    );
    let removed = payload
        .get("removed")
        .and_then(Value::as_array)
        .expect("payload.removed should be an array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(removed, vec!["a".to_string(), "b".to_string()]);
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "command should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_empty_config(config_path: &Path, storage_root: &Path) {
    let contents = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n\nskills = []\n",
        toml_escape_path(storage_root)
    );
    fs::write(config_path, contents).expect("write empty config");
}

fn write_config(config_path: &Path, storage_root: &Path, target_root: &Path, ids: &[&str]) {
    let repo_root = config_path
        .parent()
        .expect("config has parent")
        .join("mock-repo");
    fs::create_dir_all(&repo_root).expect("create mock repo");
    fs::create_dir_all(storage_root).expect("create storage root");
    fs::create_dir_all(target_root).expect("create target root");

    let mut contents = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n\n",
        toml_escape_path(storage_root)
    );
    for id in ids {
        contents.push_str(&format!(
            "[[skills]]\nid = \"{}\"\n\n[skills.source]\nrepo = \"{}\"\nsubpath = \".\"\nref = \"main\"\n\n[skills.install]\nmode = \"symlink\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{}\"\n\n[skills.verify]\nenabled = true\nchecks = [\"path-exists\", \"target-resolves\", \"is-symlink\"]\n\n[skills.safety]\nno_exec_metadata_only = false\n\n",
            toml_escape_str(id),
            toml_escape_path(&repo_root),
            toml_escape_path(target_root),
        ));
    }

    fs::write(config_path, contents).expect("write config");
}

fn write_lock_snapshot(config_path: &Path) {
    let loaded = load_from_file(config_path, LoadOptions { strict: false }).expect("load config");
    let config_dir = config_path.parent().unwrap_or(Path::new("."));
    let lock = build_lock_from_config(&loaded.config, config_dir, &HashMap::new())
        .expect("build lock snapshot");
    let lock_path = lock_path_for_config(config_path);
    write_lock_file(&lock_path, &lock).expect("write lock");
}

fn read_skill_ids(config_path: &Path) -> Vec<String> {
    let text = fs::read_to_string(config_path).expect("read config");
    let parsed: Value = toml::from_str(&text).expect("parse config");
    parsed
        .get("skills")
        .and_then(Value::as_array)
        .map(|skills| {
            skills
                .iter()
                .filter_map(|skill| {
                    skill
                        .as_table()
                        .and_then(|table| table.get("id"))
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
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
    fs::write(skill_dir.join("README.md"), "demo").expect("write skill README");
}

fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

fn toml_escape_str(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
