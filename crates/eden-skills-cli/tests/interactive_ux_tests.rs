mod common;

use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use dialoguer::console::{measure_text_width, set_colors_enabled, set_colors_enabled_stderr};
use eden_skills_cli::ui::{
    configure_color_output, ColorWhen, SkillSelectItem, SkillSelectTheme, UiContext,
};
use tempfile::tempdir;
use toml::Value;

#[test]
fn remove_without_args_selects_expected_skills_from_test_indices() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(
        &config_path,
        &storage_root,
        &target_root,
        &["alpha", "beta", "gamma"],
    );

    let output = common::eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "0,2")
        .env("EDEN_SKILLS_TEST_CONFIRM", "y")
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run interactive remove");
    common::assert_success(&output);

    let remaining = read_skill_ids(&config_path);
    assert_eq!(remaining, vec!["beta".to_string()]);
}

#[test]
fn install_uses_test_indices_for_multi_skill_selection() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("interactive-install");
    write_skill(
        &repo_dir.join("skills/alpha/SKILL.md"),
        "alpha-skill",
        "Alpha",
    );
    write_skill(&repo_dir.join("skills/beta/SKILL.md"), "beta-skill", "Beta");
    write_skill(
        &repo_dir.join("skills/gamma/SKILL.md"),
        "gamma-skill",
        "Gamma",
    );

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_CONFIRM", "n")
        .env("EDEN_SKILLS_TEST_SKILL_INPUT", "0,1")
        .args(["install", "./interactive-install", "--config"])
        .arg(&config_path)
        .output()
        .expect("run interactive install");
    common::assert_success(&output);

    let mut installed = read_skill_ids(&config_path);
    installed.sort();
    assert_eq!(
        installed,
        vec!["alpha-skill".to_string(), "beta-skill".to_string()]
    );
}

#[test]
fn install_all_flag_bypasses_interactive_test_input() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("install-all");
    write_skill(
        &repo_dir.join("skills/alpha/SKILL.md"),
        "alpha-skill",
        "Alpha",
    );
    write_skill(&repo_dir.join("skills/beta/SKILL.md"), "beta-skill", "Beta");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_SKILL_INPUT", "interrupt")
        .args(["install", "./install-all", "--all", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install --all");
    common::assert_success(&output);

    let mut installed = read_skill_ids(&config_path);
    installed.sort();
    assert_eq!(
        installed,
        vec!["alpha-skill".to_string(), "beta-skill".to_string()]
    );
}

#[test]
fn install_skill_flag_bypasses_interactive_test_input() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("install-skill");
    write_skill(
        &repo_dir.join("skills/alpha/SKILL.md"),
        "alpha-skill",
        "Alpha",
    );
    write_skill(&repo_dir.join("skills/beta/SKILL.md"), "beta-skill", "Beta");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_SKILL_INPUT", "interrupt")
        .args([
            "install",
            "./install-skill",
            "--skill",
            "beta-skill",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install --skill");
    common::assert_success(&output);

    assert_eq!(read_skill_ids(&config_path), vec!["beta-skill".to_string()]);
}

#[test]
fn install_single_discovered_skill_bypasses_interactive_test_input() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let repo_dir = temp.path().join("install-single");
    write_skill(&repo_dir.join("SKILL.md"), "solo-skill", "Solo");

    let config_path = temp.path().join("skills.toml");
    let output = common::eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_SKILL_INPUT", "interrupt")
        .args(["install", "./install-single", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install single");
    common::assert_success(&output);

    assert_eq!(read_skill_ids(&config_path), vec!["solo-skill".to_string()]);
}

#[test]
fn remove_test_input_interrupt_cancels_without_modifying_config() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(
        &config_path,
        &storage_root,
        &target_root,
        &["alpha", "beta"],
    );

    let output = common::eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "interrupt")
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run interrupted interactive remove");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("◆  Remove canceled"), "stdout={stdout}");
    assert_eq!(
        read_skill_ids(&config_path),
        vec!["alpha".to_string(), "beta".to_string()]
    );
}

#[test]
fn remove_star_input_is_not_treated_as_wildcard() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(
        &config_path,
        &storage_root,
        &target_root,
        &["alpha", "beta", "gamma"],
    );

    let output = common::eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "*")
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run remove with star test input");
    assert_eq!(
        output.status.code(),
        Some(2),
        "star input should fail, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid interactive selection index"),
        "stderr={stderr}"
    );
    assert_eq!(
        read_skill_ids(&config_path),
        vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()]
    );
}

#[test]
fn remove_explicit_ids_ignore_interactive_test_input() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(
        &config_path,
        &storage_root,
        &target_root,
        &["alpha", "beta", "gamma"],
    );

    let output = common::eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "0,2")
        .env("EDEN_SKILLS_TEST_CONFIRM", "y")
        .args(["remove", "beta", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run explicit remove");
    common::assert_success(&output);

    assert_eq!(
        read_skill_ids(&config_path),
        vec!["alpha".to_string(), "gamma".to_string()]
    );
}

#[test]
fn remove_selection_declined_confirmation_keeps_config_unchanged() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("targets");
    write_config(
        &config_path,
        &storage_root,
        &target_root,
        &["alpha", "beta", "gamma"],
    );

    let output = common::eden_command(&home_dir)
        .env_remove("CI")
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .env("EDEN_SKILLS_TEST_REMOVE_INPUT", "0,2")
        .env("EDEN_SKILLS_TEST_CONFIRM", "n")
        .args(["remove", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run declined interactive remove");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Remove cancelled"), "stdout={stdout}");
    assert_eq!(
        read_skill_ids(&config_path),
        vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()]
    );
}

#[test]
fn skill_select_theme_shows_description_for_active_unchecked_item() {
    let _guard = lock_test_env();
    std::env::set_var("COLUMNS", "80");

    let items = [SkillSelectItem {
        name: "alpha-skill",
        description: "Alpha description",
    }];
    let theme = SkillSelectTheme::new(&items, false);

    let inactive = theme.format_item_preview("alpha-skill", false, false);
    let active = theme.format_item_preview("alpha-skill", false, true);

    assert!(!inactive.contains('('), "inactive={inactive}");
    assert!(active.contains("(Alpha description)"), "active={active}");
}

#[test]
fn skill_select_theme_keeps_description_visible_for_checked_item() {
    let _guard = lock_test_env();
    std::env::set_var("COLUMNS", "80");

    let items = [SkillSelectItem {
        name: "alpha-skill",
        description: "Alpha description",
    }];
    let theme = SkillSelectTheme::new(&items, false);
    let rendered = theme.format_item_preview("alpha-skill", true, false);

    assert!(
        rendered.contains("(Alpha description)"),
        "rendered={rendered}"
    );
}

#[test]
fn skill_select_theme_truncates_description_to_57_chars_before_ellipsis() {
    let _guard = lock_test_env();
    std::env::set_var("COLUMNS", "120");

    let description = "123456789012345678901234567890123456789012345678901234567890";
    let items = [SkillSelectItem {
        name: "alpha-skill",
        description,
    }];
    let theme = SkillSelectTheme::new(&items, false);
    let rendered = theme.format_item_preview("alpha-skill", false, true);

    assert!(
        rendered.contains("(123456789012345678901234567890123456789012345678901234567...)"),
        "rendered={rendered}"
    );
}

#[test]
fn skill_select_theme_further_truncates_to_avoid_soft_wrap() {
    let _guard = lock_test_env();
    std::env::set_var("COLUMNS", "28");

    let items = [SkillSelectItem {
        name: "alpha-skill",
        description: "Alpha description needs truncation",
    }];
    let theme = SkillSelectTheme::new(&items, false);
    let rendered = theme.format_item_preview("alpha-skill", false, true);

    assert!(rendered.contains("..."), "rendered={rendered}");
    assert!(measure_text_width(&rendered) <= 28, "rendered={rendered}");
}

#[test]
fn skill_select_theme_omits_empty_description_parentheses() {
    let items = [SkillSelectItem {
        name: "plain-skill",
        description: "",
    }];
    let theme = SkillSelectTheme::new(&items, false);
    let rendered = theme.format_item_preview("plain-skill", true, true);

    assert!(!rendered.contains('('), "rendered={rendered}");
}

#[test]
fn skill_select_theme_formats_install_prompt_text() {
    let items = [SkillSelectItem {
        name: "alpha-skill",
        description: "Alpha",
    }];
    let theme = SkillSelectTheme::new(&items, false);
    let rendered = theme.format_prompt_line("Select skills to install");

    assert!(
        rendered.contains("Select skills to install") && rendered.contains("(space to toggle)"),
        "rendered={rendered}"
    );
}

#[test]
fn skill_select_theme_formats_remove_prompt_text() {
    let items = [SkillSelectItem {
        name: "alpha-skill",
        description: "",
    }];
    let theme = SkillSelectTheme::new(&items, false);
    let rendered = theme.format_prompt_line("Select skills to remove");

    assert!(
        rendered.contains("Select skills to remove") && rendered.contains("(space to toggle)"),
        "rendered={rendered}"
    );
}

#[test]
fn skill_select_theme_renders_overflow_indicators_at_top_and_bottom() {
    let items = [
        SkillSelectItem {
            name: "skill-1",
            description: "",
        },
        SkillSelectItem {
            name: "skill-2",
            description: "",
        },
        SkillSelectItem {
            name: "skill-3",
            description: "",
        },
        SkillSelectItem {
            name: "skill-4",
            description: "",
        },
        SkillSelectItem {
            name: "skill-5",
            description: "",
        },
        SkillSelectItem {
            name: "skill-6",
            description: "",
        },
    ];
    let selected = vec![false; items.len()];
    let theme = SkillSelectTheme::new(&items, false);
    let frame = theme.render_frame(
        Some("Found 6 skills"),
        "Select skills to install",
        &items,
        &selected,
        3,
        3,
    );

    assert!(
        frame.iter().any(|line| line.trim() == "..."),
        "frame={frame:?}"
    );
    assert_eq!(
        frame.iter().filter(|line| line.trim() == "...").count(),
        2,
        "frame={frame:?}"
    );
}

#[test]
fn skill_select_theme_styles_active_and_checked_states_without_bold() {
    let _guard = lock_test_env();
    set_colors_enabled(true);
    set_colors_enabled_stderr(true);

    let items = [SkillSelectItem {
        name: "alpha-skill",
        description: "Alpha description",
    }];
    let theme = SkillSelectTheme::new(&items, true);
    let active = theme.format_item_preview("alpha-skill", false, true);
    let checked = theme.format_item_preview("alpha-skill", true, false);

    assert!(active.contains("\u{1b}[36m"), "active={active}");
    assert!(checked.contains("\u{1b}[32m"), "checked={checked}");
    assert!(
        active.contains("\u{1b}[36m◻\u{1b}[0m alpha-skill"),
        "active={active}"
    );
    assert!(!active.contains("\u{1b}[1m"), "active={active}");
    assert!(!checked.contains("\u{1b}[1m"), "checked={checked}");
}

#[test]
fn ui_context_signal_cancelled_line_uses_red_diamond() {
    let _guard = lock_test_env();
    configure_color_output(ColorWhen::Always, false);
    let ui = UiContext::from_env(false);
    let rendered = ui.signal_cancelled_line("Install");

    assert!(
        rendered.contains("◆  Install canceled"),
        "rendered={rendered}"
    );
    assert!(rendered.contains("\u{1b}[31m"), "rendered={rendered}");
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
        common::toml_escape_path(storage_root)
    );
    for id in ids {
        contents.push_str(&format!(
            "[[skills]]\nid = \"{}\"\n\n[skills.source]\nrepo = \"{}\"\nsubpath = \".\"\nref = \"main\"\n\n[skills.install]\nmode = \"symlink\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{}\"\n\n[skills.verify]\nenabled = true\nchecks = [\"path-exists\", \"target-resolves\", \"is-symlink\"]\n\n[skills.safety]\nno_exec_metadata_only = false\n\n",
            common::toml_escape_string(id),
            common::toml_escape_path(&repo_root),
            common::toml_escape_path(target_root),
        ));
    }

    fs::write(config_path, contents).expect("write config");
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

fn test_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_test_env() -> MutexGuard<'static, ()> {
    test_env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
