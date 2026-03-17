mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::{Mutex, OnceLock};

use eden_skills_cli::ui::{configure_color_output, ColorWhen, UiContext};
use tempfile::tempdir;

#[test]
fn tm_p297_007_cli_cargo_toml_enables_custom_styling_feature() {
    let cargo_toml = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read crates/eden-skills-cli/Cargo.toml");
    assert!(
        cargo_toml.contains("custom_styling"),
        "CLI Cargo.toml must enable comfy-table custom_styling feature, Cargo.toml={cargo_toml}"
    );
}

#[test]
fn tm_p297_008_list_table_headers_render_bold_when_colors_are_enabled() {
    let fixture = setup_list_fixture();
    let output = run_command_with_config(&fixture, "always", true, &["list"]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let header_line = stdout
        .lines()
        .find(|line| line.contains("Skill") && line.contains("Agents"))
        .unwrap_or_else(|| {
            panic!("expected list output to contain table header row, stdout={stdout}")
        });

    assert!(
        header_line.contains("\u{1b}[1m"),
        "table headers should contain ANSI bold sequence when colors are enabled, line={header_line:?} stdout={stdout}"
    );
}

#[test]
fn tm_p297_009_list_skill_id_cells_render_bold_magenta_when_colors_are_enabled() {
    let fixture = setup_list_fixture();
    let output = run_command_with_config(&fixture, "always", true, &["list"]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let skill_line = stdout
        .lines()
        .find(|line| line.contains("alpha-skill"))
        .unwrap_or_else(|| {
            panic!("expected list output to contain alpha-skill row, stdout={stdout}")
        });

    assert!(
        skill_line.contains("\u{1b}[1m") && skill_line.contains("\u{1b}[35m"),
        "skill id cell should contain bold and magenta ANSI sequences when colors are enabled, line={skill_line:?} stdout={stdout}"
    );
}

#[test]
fn tm_p297_010_ui_context_styles_status_cells_by_semantic_category() {
    let _guard = test_env_lock();
    std::env::set_var("EDEN_SKILLS_FORCE_TTY", "1");
    std::env::remove_var("CI");
    std::env::remove_var("NO_COLOR");
    std::env::remove_var("FORCE_COLOR");

    configure_color_output(ColorWhen::Always, false);
    let ui = UiContext::from_env(false);
    let up_to_date = ui.styled_status("up-to-date");
    let failed = ui.styled_status("failed");

    assert!(
        up_to_date.contains("\u{1b}[32m"),
        "up-to-date status should use green styling, rendered={up_to_date:?}"
    );
    assert!(
        failed.contains("\u{1b}[31m"),
        "failed status should use red styling, rendered={failed:?}"
    );
}

#[test]
fn tm_p297_011_styled_table_cells_keep_visible_column_alignment() {
    let _guard = test_env_lock();
    std::env::set_var("EDEN_SKILLS_FORCE_TTY", "1");
    std::env::remove_var("CI");
    std::env::remove_var("NO_COLOR");
    std::env::remove_var("FORCE_COLOR");

    configure_color_output(ColorWhen::Always, false);
    let ui = UiContext::from_env(false);
    let mut table = ui.table(&["Skill", "Status", "Path"]);
    table.add_row(vec![
        ui.styled_skill_id("alpha-skill"),
        ui.styled_status("new commit"),
        ui.styled_path("/tmp/cache/alpha-skill"),
    ]);
    table.add_row(vec![
        ui.styled_skill_id("beta-skill"),
        ui.styled_status("failed"),
        ui.styled_path("/tmp/cache/beta-skill"),
    ]);

    let rendered = table.to_string();
    let visible_lines = rendered
        .lines()
        .map(strip_ansi)
        .filter(|line| line.starts_with('│'))
        .collect::<Vec<_>>();
    let expected_positions = separator_positions(&visible_lines[0]);

    assert!(
        visible_lines
            .iter()
            .all(|line| separator_positions(line) == expected_positions),
        "styled cells should preserve visible column alignment, rendered={rendered}"
    );
}

#[test]
fn tm_p297_012_non_tty_list_output_contains_no_ansi_sequences() {
    let fixture = setup_list_fixture();
    let output = run_command_with_config(&fixture, "auto", false, &["list"]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !has_ansi_codes(&stdout),
        "non-TTY table output should remain plain text, stdout={stdout}"
    );
}

#[test]
fn tm_p297_057_install_help_uses_colored_headers_literals_and_placeholders() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let output = common::eden_command(&home_dir)
        .args(["--color", "always", "install", "--help"])
        .output()
        .expect("run install --help");
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\u{1b}[1m\u{1b}[32mUsage:")
            && stdout.contains("\u{1b}[1m\u{1b}[32mOptions:")
            && stdout.contains("\u{1b}[1m\u{1b}[32mQuick Management:"),
        "help headers should be bold green, stdout={stdout}"
    );
    assert!(
        stdout.contains("\u{1b}[1m\u{1b}[36meden-skills install")
            && stdout.contains("\u{1b}[1m\u{1b}[36m--config"),
        "help literals should be bold cyan, stdout={stdout}"
    );
    assert!(
        stdout.contains("\u{1b}[35m<SOURCE>") && stdout.contains("\u{1b}[35m<CONFIG>"),
        "help placeholders should be magenta, stdout={stdout}"
    );
}

#[test]
fn tm_p297_058_list_uses_path_column_with_repo_cache_paths() {
    let fixture = setup_home_relative_list_fixture();
    let output = run_command_with_config(&fixture, "never", true, &["list"]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Source") && !stdout.contains("Path"),
        "list table should expose Source column instead of Path, stdout={stdout}"
    );
    assert!(
        stdout.contains("vercel-labs/agent-skills (skills/alpha-skill)"),
        "list source column should show owner/repo (subpath), stdout={stdout}"
    );
}

#[test]
fn tm_p297_059_list_agents_column_truncates_after_five_with_yellow_suffix() {
    let fixture = setup_agents_overflow_fixture();
    let output = run_command_with_config(&fixture, "always", true, &["list"]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let skill_line = stdout
        .lines()
        .find(|line| line.contains("overflow-skill"))
        .unwrap_or_else(|| panic!("expected overflow-skill row in list output, stdout={stdout}"));

    assert!(
        skill_line.contains("claude-code, cursor, codex, windsurf, opencode"),
        "agents column should keep the first five agent labels, line={skill_line:?}"
    );
    assert!(
        skill_line.contains("+2 more") && skill_line.contains("\u{1b}[33m"),
        "agents overflow suffix should be yellow and report hidden count, line={skill_line:?} stdout={stdout}"
    );
}

struct Fixture {
    temp: tempfile::TempDir,
    home_dir: PathBuf,
    config_path: PathBuf,
}

struct SkillTarget {
    agent: String,
    path: Option<String>,
}

struct SkillEntry {
    id: String,
    repo: String,
    subpath: String,
    mode: String,
    metadata_only: bool,
    targets: Vec<SkillTarget>,
}

fn setup_list_fixture() -> Fixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let storage_root = temp.path().join("storage");
    let config_path = temp.path().join("skills.toml");
    let skills = default_list_skills(temp.path());

    write_skills_config(&config_path, &storage_root, &skills);
    Fixture {
        temp,
        home_dir,
        config_path,
    }
}

fn setup_home_relative_list_fixture() -> Fixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let config_path = temp.path().join("skills.toml");
    let skills = default_list_skills(temp.path());
    write_skills_config_with_storage_root(&config_path, "~/.eden-skills/skills", &skills);

    Fixture {
        temp,
        home_dir,
        config_path,
    }
}

fn setup_agents_overflow_fixture() -> Fixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let config_path = temp.path().join("skills.toml");
    let skills = vec![SkillEntry {
        id: "overflow-skill".to_string(),
        repo: "https://github.com/vercel-labs/agent-skills.git".to_string(),
        subpath: "skills/overflow-skill".to_string(),
        mode: "symlink".to_string(),
        metadata_only: false,
        targets: vec![
            SkillTarget {
                agent: "claude-code".to_string(),
                path: None,
            },
            SkillTarget {
                agent: "cursor".to_string(),
                path: None,
            },
            SkillTarget {
                agent: "codex".to_string(),
                path: None,
            },
            SkillTarget {
                agent: "windsurf".to_string(),
                path: None,
            },
            SkillTarget {
                agent: "opencode".to_string(),
                path: None,
            },
            SkillTarget {
                agent: "gemini-cli".to_string(),
                path: None,
            },
            SkillTarget {
                agent: "continue".to_string(),
                path: None,
            },
        ],
    }];
    write_skills_config_with_storage_root(&config_path, "~/.eden-skills/skills", &skills);

    Fixture {
        temp,
        home_dir,
        config_path,
    }
}

fn default_list_skills(base_dir: &Path) -> Vec<SkillEntry> {
    let custom_target = base_dir.join("custom-agent");
    vec![
        SkillEntry {
            id: "alpha-skill".to_string(),
            repo: "https://github.com/vercel-labs/agent-skills.git".to_string(),
            subpath: "skills/alpha-skill".to_string(),
            mode: "symlink".to_string(),
            metadata_only: false,
            targets: vec![
                SkillTarget {
                    agent: "claude-code".to_string(),
                    path: None,
                },
                SkillTarget {
                    agent: "cursor".to_string(),
                    path: None,
                },
            ],
        },
        SkillEntry {
            id: "beta-skill".to_string(),
            repo: "https://github.com/vercel-labs/agent-skills".to_string(),
            subpath: "skills/beta-skill".to_string(),
            mode: "copy".to_string(),
            metadata_only: true,
            targets: vec![SkillTarget {
                agent: "claude-code".to_string(),
                path: None,
            }],
        },
        SkillEntry {
            id: "gamma-skill".to_string(),
            repo: "https://github.com/user/custom-skills.git".to_string(),
            subpath: "skills/gamma-skill".to_string(),
            mode: "symlink".to_string(),
            metadata_only: false,
            targets: vec![SkillTarget {
                agent: "custom".to_string(),
                path: Some(custom_target.display().to_string()),
            }],
        },
    ]
}

fn write_skills_config(config_path: &Path, storage_root: &Path, skills: &[SkillEntry]) {
    write_skills_config_with_storage_root(
        config_path,
        &common::toml_escape_path(storage_root),
        skills,
    );
}

fn write_skills_config_with_storage_root(
    config_path: &Path,
    storage_root: &str,
    skills: &[SkillEntry],
) {
    let mut config = format!("version = 1\n\n[storage]\nroot = \"{storage_root}\"\n");

    for skill in skills {
        config.push_str("\n[[skills]]\n");
        config.push_str(&format!(
            "id = \"{}\"\n\n",
            common::toml_escape_string(&skill.id)
        ));
        config.push_str("[skills.source]\n");
        config.push_str(&format!(
            "repo = \"{}\"\n",
            common::toml_escape_string(&skill.repo)
        ));
        config.push_str(&format!(
            "subpath = \"{}\"\n",
            common::toml_escape_string(&skill.subpath)
        ));
        config.push_str("ref = \"main\"\n\n");
        config.push_str("[skills.install]\n");
        config.push_str(&format!(
            "mode = \"{}\"\n",
            common::toml_escape_string(&skill.mode)
        ));
        for target in &skill.targets {
            config.push_str("\n[[skills.targets]]\n");
            config.push_str(&format!(
                "agent = \"{}\"\n",
                common::toml_escape_string(&target.agent)
            ));
            if let Some(path) = &target.path {
                config.push_str(&format!(
                    "path = \"{}\"\n",
                    common::toml_escape_string(path)
                ));
            }
        }
        config.push_str("\n[skills.verify]\n");
        config.push_str("enabled = true\n");
        config.push_str("checks = [\"path-exists\"]\n\n");
        config.push_str("[skills.safety]\n");
        config.push_str(&format!(
            "no_exec_metadata_only = {}\n",
            skill.metadata_only
        ));
    }

    fs::write(config_path, config).expect("write skills config");
}

fn run_command_with_config(
    fixture: &Fixture,
    color: &str,
    force_tty: bool,
    command_args: &[&str],
) -> Output {
    let mut command = common::eden_command(&fixture.home_dir);
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
    command.arg("--color").arg(color);
    command.args(command_args);
    command.arg("--config").arg(&fixture.config_path);
    command.output().expect("run eden-skills command")
}

fn has_ansi_codes(text: &str) -> bool {
    text.as_bytes().windows(2).any(|window| window == b"\x1b[")
}

fn strip_ansi(text: &str) -> String {
    let mut stripped = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
            continue;
        }
        stripped.push(ch);
    }
    stripped
}

fn separator_positions(line: &str) -> Vec<usize> {
    line.chars()
        .enumerate()
        .filter_map(|(index, ch)| matches!(ch, '│' | '┆').then_some(index))
        .collect()
}

fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    match LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
