use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::{tempdir, TempDir};

struct Fixture {
    temp: TempDir,
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

#[test]
fn tm_p28_005_list_renders_as_table() {
    let fixture = setup_list_fixture();
    let output = run_command_with_config(&fixture, "never", true, &["list"]);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Skills") && stdout.contains("configured"),
        "list output should include skills summary header, stdout={stdout}"
    );
    assert!(
        stdout.contains("Skill")
            && stdout.contains("Mode")
            && stdout.contains("Source")
            && stdout.contains("Agents"),
        "list output should include table headers Skill/Mode/Source/Agents, stdout={stdout}"
    );
    assert!(
        stdout.contains("alpha-skill")
            && stdout.contains("beta-skill")
            && stdout.contains("gamma-skill"),
        "list output should include configured skill rows, stdout={stdout}"
    );
    assert!(
        !stdout.contains("skill id="),
        "legacy key=value list format must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_006_list_table_non_tty_degradation() {
    let fixture = setup_list_fixture();
    let output = run_command_with_config(&fixture, "auto", false, &["list"]);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains('|'),
        "non-TTY list output should use ASCII table borders, stdout={stdout}"
    );
    assert!(
        !stdout.contains('│') && !stdout.contains('┌'),
        "non-TTY list output must not use UTF-8 borders, stdout={stdout}"
    );
    assert!(
        !has_ansi_codes(&stdout),
        "non-TTY list output must not include ANSI sequences, stdout={stdout}"
    );
}

#[test]
fn tm_p28_007_list_table_json_unchanged() {
    let fixture = setup_list_fixture();
    let output = run_command_with_config(&fixture, "never", false, &["list", "--json"]);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value =
        serde_json::from_str(&stdout).unwrap_or_else(|err| panic!("invalid json: {err}"));

    assert_eq!(payload["count"].as_u64(), Some(3));
    let skills = payload["skills"].as_array().expect("skills array");
    assert_eq!(skills.len(), 3, "expected 3 skills in json payload");
    assert!(
        !stdout.contains('│') && !stdout.contains('┌'),
        "list --json must not render table borders, stdout={stdout}"
    );
}

#[test]
fn tm_p28_008_install_dry_run_renders_targets_table() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let repo_dir = setup_local_discovery_repo(
        temp.path(),
        "dry-run-repo",
        &[("preview-skill", "Preview dry-run skill")],
    );

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env_remove("EDEN_SKILLS_FORCE_TTY")
        .arg("--color")
        .arg("never")
        .args([
            "install",
            &path_as_relative_arg(&repo_dir),
            "--all",
            "--dry-run",
            "--target",
            "claude-code",
            "--target",
            "cursor",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install --dry-run");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Dry Run")
            && stdout.contains("Skill:")
            && stdout.contains("Version:")
            && stdout.contains("Source:"),
        "dry-run output should include metadata header, stdout={stdout}"
    );
    assert!(
        stdout.contains("Agent") && stdout.contains("Path") && stdout.contains("Mode"),
        "dry-run output should include targets table headers, stdout={stdout}"
    );
    assert!(
        !stdout.contains("target agent="),
        "legacy dry-run target key=value format must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_009_install_list_renders_numbered_table() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let config_path = temp.path().join("skills.toml");
    let repo_dir = setup_local_discovery_repo(
        temp.path(),
        "install-list-repo",
        &[
            ("alpha-skill", "Alpha skill"),
            ("beta-skill", "Beta skill"),
            ("gamma-skill", "Gamma skill"),
        ],
    );

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CI")
        .env_remove("EDEN_SKILLS_FORCE_TTY")
        .arg("--color")
        .arg("never")
        .args([
            "install",
            &path_as_relative_arg(&repo_dir),
            "--list",
            "--config",
        ])
        .arg(&config_path)
        .output()
        .expect("run install --list");
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains('#') && stdout.contains("Name") && stdout.contains("Description"),
        "install --list should render numbered table headers, stdout={stdout}"
    );
    assert!(
        stdout.contains("1")
            && stdout.contains("alpha-skill")
            && stdout.contains("beta-skill")
            && stdout.contains("gamma-skill"),
        "install --list should include numbered discovered skills, stdout={stdout}"
    );
    assert!(
        !stdout.contains("Skills in "),
        "legacy install --list output format must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_010_plan_table_threshold() {
    let high_fixture = setup_plan_fixture(6);
    let high_output = run_command_with_config(&high_fixture, "never", false, &["plan"]);
    assert_success(&high_output);

    let high_stdout = String::from_utf8_lossy(&high_output.stdout);
    assert!(
        high_stdout.contains("Plan") && high_stdout.contains("actions"),
        "plan output should include summary header, stdout={high_stdout}"
    );
    assert!(
        high_stdout.contains("Action")
            && high_stdout.contains("Skill")
            && high_stdout.contains("Target")
            && high_stdout.contains("Mode"),
        "plan with >5 actions should render table headers, stdout={high_stdout}"
    );

    let low_fixture = setup_plan_fixture(3);
    let low_output = run_command_with_config(&low_fixture, "never", false, &["plan"]);
    assert_success(&low_output);

    let low_stdout = String::from_utf8_lossy(&low_output.stdout);
    assert!(
        low_stdout.contains("→"),
        "plan with <=5 actions should stay in text format with arrow paths, stdout={low_stdout}"
    );
    assert!(
        !(low_stdout.contains("Action")
            && low_stdout.contains("Skill")
            && low_stdout.contains("Target")
            && low_stdout.contains("Mode")),
        "plan with <=5 actions must not render table headers, stdout={low_stdout}"
    );
}

#[test]
fn tm_p28_011_update_renders_registry_table() {
    let fixture = setup_update_fixture();
    let output = run_command_with_config(&fixture, "never", false, &["update"]);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Update") && stdout.contains("synced"),
        "update output should include action-prefix summary header, stdout={stdout}"
    );
    assert!(
        stdout.contains("Registry") && stdout.contains("Status") && stdout.contains("Detail"),
        "update output should render registry result table, stdout={stdout}"
    );
    assert!(
        !stdout.contains("registry sync:"),
        "legacy update summary format must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p28_029_non_tty_tables_use_ascii_borders() {
    let fixture = setup_plan_fixture(6);
    let output = run_command_with_config(&fixture, "never", false, &["plan"]);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains('|'),
        "non-TTY plan table should use ASCII borders, stdout={stdout}"
    );
    assert!(
        !stdout.contains('│') && !stdout.contains('┌'),
        "non-TTY plan table must not use UTF-8 borders, stdout={stdout}"
    );
}

#[test]
fn tm_p28_030_color_never_disables_table_styling() {
    let fixture = setup_list_fixture();
    let output = run_command_with_config(&fixture, "never", true, &["list"]);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Skill") && (stdout.contains('│') || stdout.contains('|')),
        "table structure should remain with --color never, stdout={stdout}"
    );
    assert!(
        !has_ansi_codes(&stdout),
        "--color never must disable ANSI styling, stdout={stdout}"
    );
}

#[test]
fn tm_p28_031_json_mode_never_renders_tables() {
    let list_fixture = setup_list_fixture();
    let list_output = run_command_with_config(&list_fixture, "always", true, &["list", "--json"]);
    assert_success(&list_output);
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let list_payload: Value =
        serde_json::from_str(&list_stdout).unwrap_or_else(|err| panic!("invalid json: {err}"));
    assert!(
        list_payload.get("skills").is_some(),
        "list json must include skills"
    );
    assert!(
        !list_stdout.contains('│') && !list_stdout.contains('┌') && !list_stdout.contains("|"),
        "list --json output must not contain table fragments, stdout={list_stdout}"
    );

    let plan_fixture = setup_plan_fixture(6);
    let plan_output = run_command_with_config(&plan_fixture, "always", true, &["plan", "--json"]);
    assert_success(&plan_output);
    let plan_stdout = String::from_utf8_lossy(&plan_output.stdout);
    let plan_payload: Value =
        serde_json::from_str(&plan_stdout).unwrap_or_else(|err| panic!("invalid json: {err}"));
    assert!(plan_payload.is_array(), "plan json should be an array");
    assert!(
        !plan_stdout.contains('│') && !plan_stdout.contains('┌') && !plan_stdout.contains("|"),
        "plan --json output must not contain table fragments, stdout={plan_stdout}"
    );
}

fn setup_list_fixture() -> Fixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let storage_root = temp.path().join("storage");
    let config_path = temp.path().join("skills.toml");
    let custom_target = temp.path().join("custom-agent");

    let skills = vec![
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
    ];

    write_skills_config(&config_path, &storage_root, &skills);
    Fixture {
        temp,
        home_dir,
        config_path,
    }
}

fn setup_plan_fixture(skill_count: usize) -> Fixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let origin_repo = init_git_repo(
        temp.path(),
        "plan-origin",
        &[("packages/browser/README.md", "seed\n")],
    );
    let repo_url = as_file_url(&origin_repo);
    let storage_root = temp.path().join("storage");
    let config_path = temp.path().join("skills.toml");

    let mut skills = Vec::new();
    for index in 0..skill_count {
        skills.push(SkillEntry {
            id: format!("plan-skill-{index}"),
            repo: repo_url.clone(),
            subpath: "packages/browser".to_string(),
            mode: "symlink".to_string(),
            metadata_only: false,
            targets: vec![SkillTarget {
                agent: "custom".to_string(),
                path: Some(temp.path().join("agent-target").display().to_string()),
            }],
        });
    }
    write_skills_config(&config_path, &storage_root, &skills);

    Fixture {
        temp,
        home_dir,
        config_path,
    }
}

fn setup_update_fixture() -> Fixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let config_path = temp.path().join("skills.toml");

    let skill_repo = init_git_repo(
        temp.path(),
        "skill-origin",
        &[("packages/browser/README.md", "seed\n")],
    );
    let official_registry = init_git_repo(
        temp.path(),
        "registry-official",
        &[("manifest.toml", "format_version = 1\nname = \"official\"\n")],
    );
    let forge_registry = init_git_repo(
        temp.path(),
        "registry-forge",
        &[("manifest.toml", "format_version = 1\nname = \"forge\"\n")],
    );

    let config = format!(
        r#"version = 1

[storage]
root = "{storage_root}"

[registries]
official = {{ url = "{official_url}", priority = 100 }}
forge = {{ url = "{forge_url}", priority = 10 }}

[[skills]]
id = "registry-skill"

[skills.source]
repo = "{skill_url}"
subpath = "packages/browser"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "custom"
path = "{target_root}"

[skills.verify]
enabled = true
checks = ["path-exists"]

[skills.safety]
no_exec_metadata_only = false
"#,
        storage_root = toml_escape_path(&storage_root),
        official_url = toml_escape_str(&as_file_url(&official_registry)),
        forge_url = toml_escape_str(&as_file_url(&forge_registry)),
        skill_url = toml_escape_str(&as_file_url(&skill_repo)),
        target_root = toml_escape_path(&target_root),
    );
    fs::write(&config_path, config).expect("write update fixture config");

    Fixture {
        temp,
        home_dir,
        config_path,
    }
}

fn write_skills_config(config_path: &Path, storage_root: &Path, skills: &[SkillEntry]) {
    let mut config = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n",
        toml_escape_path(storage_root)
    );

    for skill in skills {
        config.push_str("\n[[skills]]\n");
        config.push_str(&format!("id = \"{}\"\n\n", toml_escape_str(&skill.id)));
        config.push_str("[skills.source]\n");
        config.push_str(&format!("repo = \"{}\"\n", toml_escape_str(&skill.repo)));
        config.push_str(&format!(
            "subpath = \"{}\"\n",
            toml_escape_str(&skill.subpath)
        ));
        config.push_str("ref = \"main\"\n\n");
        config.push_str("[skills.install]\n");
        config.push_str(&format!("mode = \"{}\"\n", toml_escape_str(&skill.mode)));
        for target in &skill.targets {
            config.push_str("\n[[skills.targets]]\n");
            config.push_str(&format!("agent = \"{}\"\n", toml_escape_str(&target.agent)));
            if let Some(path) = &target.path {
                config.push_str(&format!("path = \"{}\"\n", toml_escape_str(path)));
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
