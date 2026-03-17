mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use serde_json::Value;
use tempfile::tempdir;

struct Fixture {
    temp: tempfile::TempDir,
    home_dir: PathBuf,
    config_path: PathBuf,
}

#[test]
fn tm_p298_001_list_header_uses_source_column() {
    let fixture = setup_fixture();
    let output = run_list(&fixture, "never", false, &[]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Source") && !stdout.contains("Path"),
        "list output should expose Source header instead of Path, stdout={stdout}"
    );
}

#[test]
fn tm_p298_002_list_source_column_renders_github_repo_display() {
    let fixture = setup_fixture();
    let output = run_list(&fixture, "never", true, &[]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("vercel-labs/agent-skills (skills/react-best-practices)"),
        "remote sources should render owner/repo (subpath), stdout={stdout}"
    );
}

#[test]
fn tm_p298_003_list_source_column_renders_home_abbreviated_local_repo_display() {
    let fixture = setup_fixture();
    let output = run_list(&fixture, "never", true, &[]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("~/local-skills (my-skill)"),
        "local sources should abbreviate HOME in Source display, stdout={stdout}"
    );
}

#[test]
fn tm_p298_004_list_source_cells_use_cyan_ansi_when_colors_are_enabled() {
    let fixture = setup_fixture();
    let output = run_list(&fixture, "always", true, &[]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\u{1b}[36mvercel-labs/agent-skills (skills/react-best-practices)"),
        "source cells should be cyan when colors are enabled, stdout={stdout}"
    );
}

#[test]
fn tm_p298_005_list_source_cells_are_plain_text_with_color_never() {
    let fixture = setup_fixture();
    let output = run_list(&fixture, "never", true, &[]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("vercel-labs/agent-skills (skills/react-best-practices)"),
        "plain-text source display should still be present, stdout={stdout}"
    );
    assert!(
        !stdout.contains("\u{1b}[36m"),
        "color-never output must not contain cyan ANSI sequences, stdout={stdout}"
    );
}

#[test]
fn tm_p298_006_list_json_schema_preserves_source_object() {
    let fixture = setup_fixture();
    let output = run_list(&fixture, "never", false, &["--json"]);
    common::assert_success(&output);

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid list json");
    assert_eq!(payload["count"].as_u64(), Some(2));

    let skills = payload["skills"].as_array().expect("skills array");
    assert_eq!(skills.len(), 2);

    assert_eq!(
        skills[0]["source"]["repo"].as_str(),
        Some("https://github.com/vercel-labs/agent-skills.git")
    );
    assert_eq!(
        skills[0]["source"]["subpath"].as_str(),
        Some("skills/react-best-practices")
    );
    assert_eq!(skills[0]["source"]["ref"].as_str(), Some("main"));

    assert_eq!(
        skills[1]["source"]["repo"].as_str(),
        fixture.home_dir.join("local-skills").to_str()
    );
    assert_eq!(skills[1]["source"]["subpath"].as_str(), Some("my-skill"));
    assert_eq!(skills[1]["source"]["ref"].as_str(), Some("main"));
}

fn setup_fixture() -> Fixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");
    fs::create_dir_all(home_dir.join("local-skills")).expect("create local repo dir");

    let config_path = temp.path().join("skills.toml");
    write_config(&config_path, &home_dir);

    Fixture {
        temp,
        home_dir,
        config_path,
    }
}

fn write_config(config_path: &Path, home_dir: &Path) {
    let storage_root = config_path.parent().expect("config parent").join("storage");
    let remote_target = config_path
        .parent()
        .expect("config parent")
        .join("remote-targets");
    let local_target = config_path
        .parent()
        .expect("config parent")
        .join("local-targets");
    let local_repo = home_dir.join("local-skills");
    let config = format!(
        "version = 1\n\n[storage]\nroot = \"{storage_root}\"\n\n[[skills]]\nid = \"react-best-practices\"\n\n[skills.source]\nrepo = \"https://github.com/vercel-labs/agent-skills.git\"\nsubpath = \"skills/react-best-practices\"\nref = \"main\"\n\n[skills.install]\nmode = \"symlink\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{remote_target}\"\n\n[skills.verify]\nenabled = true\nchecks = [\"path-exists\"]\n\n[[skills]]\nid = \"local-skill\"\n\n[skills.source]\nrepo = \"{local_repo}\"\nsubpath = \"my-skill\"\nref = \"main\"\n\n[skills.install]\nmode = \"copy\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{local_target}\"\n\n[skills.verify]\nenabled = true\nchecks = [\"path-exists\"]\n",
        storage_root = common::toml_escape_path(&storage_root),
        remote_target = common::toml_escape_path(&remote_target),
        local_repo = common::toml_escape_path(&local_repo),
        local_target = common::toml_escape_path(&local_target),
    );
    fs::write(config_path, config).expect("write config");
}

fn run_list(fixture: &Fixture, color: &str, force_tty: bool, extra_args: &[&str]) -> Output {
    let mut command = common::eden_command(&fixture.home_dir);
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
    command
        .arg("--color")
        .arg(color)
        .arg("list")
        .args(extra_args)
        .arg("--config")
        .arg(&fixture.config_path)
        .output()
        .expect("run list")
}
