mod common;

use std::fs;
use std::path::Path;

use tempfile::tempdir;

#[test]
fn tm_p29_028_add_shows_added_line_with_abbreviated_path() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = home_dir.join(".eden-skills/skills.toml");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let init = common::eden_command(&home_dir)
        .args(["--color", "never", "init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    common::assert_success(&init);

    let output = common::eden_command(&home_dir)
        .args(["--color", "never", "add", "--config"])
        .arg(&config_path)
        .args([
            "--id",
            "ocn-add",
            "--repo",
            "https://example.com/repo.git",
            "--target",
            "claude-code",
        ])
        .output()
        .expect("run add");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("  ✓ Added 'ocn-add' to ~/.eden-skills/skills.toml"),
        "expected upgraded add line with abbreviated path, stdout={stdout}"
    );
    assert!(
        !stdout.contains("add: wrote "),
        "legacy add output must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p29_029_set_shows_updated_line_with_abbreviated_path() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = home_dir.join(".eden-skills/skills.toml");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let init = common::eden_command(&home_dir)
        .args(["--color", "never", "init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    common::assert_success(&init);

    let add = common::eden_command(&home_dir)
        .args(["--color", "never", "add", "--config"])
        .arg(&config_path)
        .args([
            "--id",
            "ocn-set",
            "--repo",
            "https://example.com/repo.git",
            "--target",
            "claude-code",
        ])
        .output()
        .expect("run add before set");
    common::assert_success(&add);

    let set = common::eden_command(&home_dir)
        .args(["--color", "never", "set", "ocn-set", "--config"])
        .arg(&config_path)
        .args(["--repo", "https://example.com/other.git"])
        .output()
        .expect("run set");
    common::assert_success(&set);

    let stdout = String::from_utf8_lossy(&set.stdout);
    assert!(
        stdout.contains("  ✓ Updated 'ocn-set' in ~/.eden-skills/skills.toml"),
        "expected upgraded set line with abbreviated path, stdout={stdout}"
    );
    assert!(
        !stdout.contains("set: wrote "),
        "legacy set output must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p29_030_config_import_shows_imported_line_with_abbreviated_path() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let from_path = temp.path().join("from.toml");
    let config_path = home_dir.join(".eden-skills/skills.toml");
    fs::create_dir_all(&home_dir).expect("create HOME");

    fs::write(
        &from_path,
        r#"
version = 1

[[skills]]
id = "imported"

[skills.source]
repo = "https://example.com/repo.git"

[[skills.targets]]
agent = "claude-code"
"#,
    )
    .expect("write import source");

    let output = common::eden_command(&home_dir)
        .args(["--color", "never", "config", "import", "--from"])
        .arg(&from_path)
        .args(["--config"])
        .arg(&config_path)
        .output()
        .expect("run config import");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("  ✓ Imported config to ~/.eden-skills/skills.toml"),
        "expected upgraded config import line with abbreviated path, stdout={stdout}"
    );
    assert!(
        !stdout.contains("config import: wrote "),
        "legacy config import output must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p29_035_ui_context_exposes_styled_path_method() {
    let ui_source = read_source_file("src/ui/context.rs");
    assert!(
        ui_source.contains("pub fn styled_path(&self, path: &str) -> String"),
        "UiContext must expose styled_path(path) API"
    );
    assert!(
        ui_source.contains("abbreviate_home_path(path)"),
        "styled_path must abbreviate HOME path with ~"
    );
    assert!(
        ui_source.contains(".cyan().to_string()"),
        "styled_path must colorize paths in cyan when colors are enabled"
    );
}

#[test]
fn tm_p29_031_no_raw_warning_eprintln_remains_in_target_files() {
    let config_ops_source = read_source_file("src/commands/config_ops.rs");
    assert!(
        !contains_raw_warning_eprintln(&config_ops_source),
        "config_ops.rs must route warning output via print_warning()"
    );

    let remove_source = read_source_file("src/commands/remove.rs");
    assert!(
        !contains_raw_warning_eprintln(&remove_source),
        "remove.rs must route warning output via print_warning()"
    );

    let common_source = read_source_file("src/commands/common.rs");
    assert!(
        !contains_raw_warning_eprintln(&common_source),
        "common.rs must not emit raw warning literals via eprintln!()"
    );
}

#[test]
fn tm_p29_032_remove_cancellation_uses_skipped_symbol_line() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = home_dir.join(".eden-skills/skills.toml");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let init = common::eden_command(&home_dir)
        .args(["--color", "never", "init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    common::assert_success(&init);

    let add = common::eden_command(&home_dir)
        .args(["--color", "never", "add", "--config"])
        .arg(&config_path)
        .args([
            "--id",
            "cancel-me",
            "--repo",
            "https://example.com/repo.git",
            "--target",
            "claude-code",
        ])
        .output()
        .expect("run add");
    common::assert_success(&add);

    let remove = common::eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_CONFIRM", "n")
        .args(["--color", "never", "remove", "cancel-me", "--config"])
        .arg(&config_path)
        .output()
        .expect("run interactive remove cancellation");
    common::assert_success(&remove);

    let stdout = String::from_utf8_lossy(&remove.stdout);
    assert!(
        stdout.contains("  · Remove cancelled"),
        "remove cancellation must use skipped symbol line, stdout={stdout}"
    );
    assert!(
        !stdout.contains("remove cancelled."),
        "legacy cancellation text must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p29_033_remove_interactive_selection_no_longer_renders_candidate_table() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = home_dir.join(".eden-skills/skills.toml");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let init = common::eden_command(&home_dir)
        .args(["--color", "never", "init", "--config"])
        .arg(&config_path)
        .output()
        .expect("run init");
    common::assert_success(&init);

    add_skill(
        &home_dir,
        &config_path,
        "skill-a",
        "https://github.com/vercel-labs/agent-skills.git",
    );
    add_skill(
        &home_dir,
        &config_path,
        "skill-b",
        "https://github.com/vercel-labs/agent-skills.git",
    );

    let remove = common::eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "0")
        .env("EDEN_SKILLS_TEST_CONFIRM", "n")
        .args(["--color", "never", "remove", "--config"])
        .arg(&config_path)
        .output()
        .expect("run interactive remove without ids");
    common::assert_success(&remove);

    let stdout = String::from_utf8_lossy(&remove.stdout);
    assert!(
        stdout.contains("Remove cancelled"),
        "interactive remove should still surface cancellation feedback, stdout={stdout}"
    );
    assert!(
        !stdout.contains("Skills   2 configured"),
        "interactive remove should not print legacy candidate heading, stdout={stdout}"
    );
    assert!(
        !stdout.contains("┌") && !stdout.contains("│"),
        "interactive remove should not render legacy candidate table borders, stdout={stdout}"
    );
    assert!(
        !stdout.contains("Source"),
        "interactive remove should not render legacy Source column, stdout={stdout}"
    );
}

fn read_source_file(relative: &str) -> String {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = crate_root.join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn contains_raw_warning_eprintln(source: &str) -> bool {
    let mut inside_eprintln = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("eprintln!(") {
            inside_eprintln = true;
            if trimmed.contains("warning:") {
                return true;
            }
            if trimmed.ends_with(");") {
                inside_eprintln = false;
            }
            continue;
        }
        if inside_eprintln {
            if trimmed.contains("warning:") {
                return true;
            }
            if trimmed.ends_with(");") {
                inside_eprintln = false;
            }
        }
    }
    false
}

fn add_skill(home_dir: &Path, config_path: &Path, skill_id: &str, repo: &str) {
    let add = common::eden_command(home_dir)
        .args(["--color", "never", "add", "--config"])
        .arg(config_path)
        .args(["--id", skill_id, "--repo", repo, "--target", "claude-code"])
        .output()
        .expect("run add");
    common::assert_success(&add);
}
