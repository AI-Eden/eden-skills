mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use eden_skills_core::source::resolve_repo_cache_root;
use serde_json::Value;
use tempfile::{tempdir, TempDir};

struct ModeAFixture {
    temp: TempDir,
    home_dir: PathBuf,
    config_path: PathBuf,
    storage_root: PathBuf,
    target_root: PathBuf,
    skill_repo: PathBuf,
}

#[test]
fn tm_p29_006_update_with_mode_a_skills_fetches_and_reports_status() {
    let fixture = setup_mode_a_fixture("symlink", false);
    let apply_output = run_command(&fixture, "never", false, &["apply"]);
    common::assert_success(&apply_output);

    commit_file(
        &fixture.skill_repo,
        "packages/browser/README.md",
        "upstream-v2\n",
        "upstream update",
    );

    let update_output = run_command(&fixture, "never", false, &["update"]);
    common::assert_success(&update_output);
    let stdout = String::from_utf8_lossy(&update_output.stdout);

    assert!(
        stdout.contains("Refresh") && stdout.contains("skills checked"),
        "update should report Mode A refresh summary, stdout={stdout}"
    );
    assert!(
        stdout.contains("mode-a-skill"),
        "refresh table should include the skill row, stdout={stdout}"
    );
    assert!(
        stdout.contains("new commit"),
        "Mode A refresh should detect upstream commit difference, stdout={stdout}"
    );
}

#[test]
fn tm_p29_007_update_without_apply_does_not_mutate_local_state() {
    let fixture = setup_mode_a_fixture("symlink", false);
    let apply_output = run_command(&fixture, "never", false, &["apply"]);
    common::assert_success(&apply_output);

    commit_file(
        &fixture.skill_repo,
        "packages/browser/README.md",
        "upstream-v2\n",
        "upstream update",
    );

    let local_repo = mode_a_repo_dir(&fixture);
    let local_head_before = git_head(&local_repo);
    let update_output = run_command(&fixture, "never", false, &["update"]);
    common::assert_success(&update_output);
    let local_head_after = git_head(&local_repo);
    let stdout = String::from_utf8_lossy(&update_output.stdout);

    assert_eq!(
        local_head_before, local_head_after,
        "update without --apply must remain fetch-only and keep HEAD unchanged"
    );
    assert!(
        stdout.contains("update --apply") || stdout.contains("eden-skills apply"),
        "read-only update with pending commits should include apply hint, stdout={stdout}"
    );
}

#[test]
fn tm_p29_008_update_apply_reconciles_skills_with_new_commits() {
    let fixture = setup_mode_a_fixture("copy", false);
    let apply_output = run_command(&fixture, "never", false, &["apply"]);
    common::assert_success(&apply_output);

    let installed_readme = fixture.target_root.join("mode-a-skill/README.md");
    let before_content = fs::read_to_string(&installed_readme).expect("read target before update");
    assert!(
        before_content.contains("seed"),
        "baseline copy target should contain seed content"
    );

    commit_file(
        &fixture.skill_repo,
        "packages/browser/README.md",
        "upstream-v2\n",
        "upstream update",
    );
    let origin_head = git_head(&fixture.skill_repo);

    let update_output = run_command(&fixture, "never", false, &["update", "--apply"]);
    common::assert_success(&update_output);
    let stdout = String::from_utf8_lossy(&update_output.stdout);

    let after_content = fs::read_to_string(&installed_readme).expect("read target after update");
    assert!(
        after_content.contains("upstream-v2"),
        "update --apply should reconcile changed copy-mode target content, stdout={stdout}"
    );
    assert!(
        stdout.contains("Install"),
        "update --apply should transition into install/reconcile output, stdout={stdout}"
    );

    let local_repo_head = git_head(&mode_a_repo_dir(&fixture));
    assert_eq!(
        local_repo_head, origin_head,
        "update --apply should advance local source repo HEAD to latest upstream commit"
    );
}

#[test]
fn tm_p29_009_update_with_no_registries_and_no_skills_shows_guidance() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    fs::write(
        &config_path,
        format!(
            "version = 1\n\n[storage]\nroot = \"{}\"\n\nskills = []\n",
            common::toml_escape_path(&storage_root)
        ),
    )
    .expect("write empty config");

    let output = run_command_raw(
        temp.path(),
        &home_dir,
        "never",
        false,
        &["update"],
        &config_path,
    );
    common::assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("no skills or registries configured"),
        "empty-state update should include combined no-sources guidance, stdout={stdout}"
    );
    assert!(
        stdout.contains("eden-skills install <owner/repo>"),
        "empty-state update should include install command hint, stdout={stdout}"
    );
}

#[test]
fn tm_p29_010_update_skill_refresh_renders_as_table() {
    let fixture = setup_mode_a_fixture("symlink", false);
    let output = run_command(&fixture, "never", false, &["update"]);
    common::assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Skill") && stdout.contains("Status"),
        "refresh results should render table headers Skill/Status, stdout={stdout}"
    );
    assert!(
        stdout.contains("mode-a-skill"),
        "refresh table should include configured skill row, stdout={stdout}"
    );
}

#[test]
fn tm_p29_011_update_skill_status_cells_are_semantically_styled() {
    let fixture = setup_mode_a_fixture("symlink", false);
    let output = run_command(&fixture, "always", true, &["update"]);
    common::assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let status_line = stdout
        .lines()
        .find(|line| {
            line.contains("missing") || line.contains("up-to-date") || line.contains("new commit")
        })
        .unwrap_or_else(|| panic!("expected at least one refresh status row, stdout={stdout}"));

    assert!(
        has_ansi_codes(status_line) && status_line.contains("\u{1b}[2m"),
        "refresh status cells should use semantic ANSI styling (missing => dim), line={status_line:?}"
    );
}

#[test]
fn tm_p29_012_update_json_includes_skills_array() {
    let fixture = setup_mode_a_fixture("symlink", false);
    let output = run_command(&fixture, "never", false, &["update", "--json"]);
    common::assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(&stdout).unwrap_or_else(|err| {
        panic!("update --json should emit valid JSON, err={err} stdout={stdout}")
    });

    let skills = payload
        .get("skills")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("update --json should include `skills` array, json={payload}"));
    assert_eq!(
        skills.len(),
        1,
        "expected one Mode A skill refresh entry in JSON output, json={payload}"
    );
    assert_eq!(
        skills[0].get("id").and_then(Value::as_str),
        Some("mode-a-skill"),
        "skill refresh JSON row should include skill id, json={payload}"
    );
    assert!(
        skills[0].get("status").and_then(Value::as_str).is_some(),
        "skill refresh JSON row should include status string, json={payload}"
    );
}

#[test]
fn tm_p29_013_update_with_registries_and_skills_shows_both_sections() {
    let fixture = setup_mode_a_fixture("symlink", true);
    let output = run_command(&fixture, "never", false, &["update"]);
    common::assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Update") && stdout.contains("registries synced"),
        "update should print registry sync section when registries are configured, stdout={stdout}"
    );
    assert!(
        stdout.contains("Registry") && stdout.contains("Detail"),
        "update should include registry result table headers, stdout={stdout}"
    );
    assert!(
        stdout.contains("Refresh") && stdout.contains("skills checked"),
        "update should append Mode A skill refresh section, stdout={stdout}"
    );
}

#[test]
fn tm_p29_014_update_skill_refresh_uses_reactor_concurrency() {
    let fixture = setup_mode_a_fixture("symlink", false);
    let output = run_command(&fixture, "never", false, &["update", "--concurrency", "0"]);

    assert_eq!(
        output.status.code(),
        Some(2),
        "invalid update concurrency should return exit code 2, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("INVALID_CONCURRENCY"),
        "update refresh should validate and use reactor concurrency overrides, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn tm_p295_034_update_mode_a_refresh_uses_repo_cache_paths() {
    let fixture = setup_mode_a_fixture("symlink", false);
    let apply_output = run_command(&fixture, "never", false, &["apply"]);
    common::assert_success(&apply_output);

    let legacy_per_skill_dir = fixture.storage_root.join("mode-a-skill");
    fs::create_dir_all(&legacy_per_skill_dir).expect("create legacy per-skill dir");
    fs::write(
        legacy_per_skill_dir.join(".git"),
        "broken legacy git marker",
    )
    .expect("write broken legacy .git");

    commit_file(
        &fixture.skill_repo,
        "packages/browser/README.md",
        "upstream-v2\n",
        "upstream update",
    );

    let update_output = run_command(&fixture, "never", false, &["update"]);
    common::assert_success(&update_output);
    let stdout = String::from_utf8_lossy(&update_output.stdout);

    assert!(
        stdout.contains("new commit"),
        "update refresh should still use repo cache checkout even when legacy per-skill dir is broken, stdout={stdout}"
    );
    assert!(
        mode_a_repo_dir(&fixture).join(".git").exists(),
        "expected repo cache checkout to remain the active Mode A refresh source"
    );
}

fn setup_mode_a_fixture(install_mode: &str, with_registries: bool) -> ModeAFixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    fs::create_dir_all(&target_root).expect("create target root");
    let config_path = temp.path().join("skills.toml");

    let skill_repo = common::init_git_repo(
        temp.path(),
        "skill-origin",
        &[("packages/browser/README.md", "seed\n")],
    );

    let mut registries: Vec<(String, String, i64)> = Vec::new();
    if with_registries {
        let official_registry = common::init_git_repo(
            temp.path(),
            "registry-official",
            &[("manifest.toml", "format_version = 1\nname = \"official\"\n")],
        );
        let forge_registry = common::init_git_repo(
            temp.path(),
            "registry-forge",
            &[("manifest.toml", "format_version = 1\nname = \"forge\"\n")],
        );
        registries.push((
            "official".to_string(),
            common::path_to_file_url(&official_registry),
            100,
        ));
        registries.push((
            "forge".to_string(),
            common::path_to_file_url(&forge_registry),
            10,
        ));
    }

    write_mode_a_config(
        &config_path,
        &storage_root,
        &common::path_to_file_url(&skill_repo),
        &target_root,
        install_mode,
        &registries,
    );

    ModeAFixture {
        temp,
        home_dir,
        config_path,
        storage_root,
        target_root,
        skill_repo,
    }
}

fn write_mode_a_config(
    config_path: &Path,
    storage_root: &Path,
    skill_repo_url: &str,
    target_root: &Path,
    install_mode: &str,
    registries: &[(String, String, i64)],
) {
    let mut config = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n",
        common::toml_escape_path(storage_root)
    );
    if !registries.is_empty() {
        config.push_str("\n[registries]\n");
        for (name, url, priority) in registries {
            config.push_str(&format!(
                "{} = {{ url = \"{}\", priority = {} }}\n",
                common::toml_escape_string(name),
                common::toml_escape_string(url),
                priority
            ));
        }
    }

    config.push_str(&format!(
        r#"

[[skills]]
id = "mode-a-skill"

[skills.source]
repo = "{skill_repo_url}"
subpath = "packages/browser"
ref = "main"

[skills.install]
mode = "{install_mode}"

[[skills.targets]]
agent = "custom"
path = "{target_root}"

[skills.verify]
enabled = true
checks = ["path-exists"]

[skills.safety]
no_exec_metadata_only = false
"#,
        skill_repo_url = common::toml_escape_string(skill_repo_url),
        install_mode = common::toml_escape_string(install_mode),
        target_root = common::toml_escape_path(target_root),
    ));
    fs::write(config_path, config).expect("write mode A config");
}

fn run_command(
    fixture: &ModeAFixture,
    color: &str,
    force_tty: bool,
    command_args: &[&str],
) -> Output {
    run_command_raw(
        fixture.temp.path(),
        &fixture.home_dir,
        color,
        force_tty,
        command_args,
        &fixture.config_path,
    )
}

fn run_command_raw(
    cwd: &Path,
    home_dir: &Path,
    color: &str,
    force_tty: bool,
    command_args: &[&str],
    config_path: &Path,
) -> Output {
    let mut command = common::eden_command(home_dir);
    command
        .current_dir(cwd)
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
    command.arg("--config").arg(config_path);
    command.output().expect("run eden-skills command")
}

fn mode_a_repo_dir(fixture: &ModeAFixture) -> PathBuf {
    resolve_repo_cache_root(
        &fixture.storage_root,
        &common::path_to_file_url(&fixture.skill_repo),
        "main",
    )
}

fn commit_file(repo: &Path, rel_path: &str, content: &str, message: &str) {
    let file_path = repo.join(rel_path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(file_path, content).expect("write commit content");
    common::run_git_cmd(repo, &["add", "."]);
    common::run_git_cmd(repo, &["commit", "-m", message]);
}

fn git_head(repo: &Path) -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .expect("read head");
    assert!(
        output.status.success(),
        "git rev-parse failed in {}: {}",
        repo.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn has_ansi_codes(text: &str) -> bool {
    text.as_bytes().windows(2).any(|window| window == b"\x1b[")
}
