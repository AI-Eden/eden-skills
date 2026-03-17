mod common;

use std::fs;
use std::path::PathBuf;
use std::process::Output;

use eden_skills_cli::commands::apply;
use serde_json::Value;
use tempfile::tempdir;

use common::{
    as_file_url, default_options, init_origin_repo, run_git_cmd, write_config,
    write_config_with_safety,
};

struct Fixture {
    temp: tempfile::TempDir,
    home_dir: PathBuf,
    config_path: PathBuf,
    storage_root: PathBuf,
}

#[test]
fn tm_p298_007_doctor_accepts_no_warning_flag() {
    let fixture = setup_warning_only_fixture();
    let output = run_doctor(&fixture, "never", true, &["--no-warning"]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no issues detected"),
        "doctor --no-warning should be a valid invocation and filter warning-only output, stdout={stdout}"
    );
}

#[test]
fn tm_p298_008_doctor_no_warning_omits_warning_findings_from_human_output() {
    let fixture = setup_error_warning_fixture();
    let output = run_doctor(&fixture, "never", true, &["--no-warning"]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("LICENSE_UNKNOWN") && !stdout.contains("warning"),
        "warning findings should be filtered from human output, stdout={stdout}"
    );
    assert!(
        stdout.contains("SOURCE_MISSING") || stdout.contains("TARGET_PATH_MISSING"),
        "error findings should remain after --no-warning, stdout={stdout}"
    );
}

#[test]
fn tm_p298_009_doctor_no_warning_omits_warning_findings_from_json_output() {
    let fixture = setup_error_warning_fixture();
    let output = run_doctor(&fixture, "never", true, &["--json", "--no-warning"]);
    common::assert_success(&output);

    let payload: Value = serde_json::from_slice(&output.stdout).expect("doctor json");
    let findings = payload["findings"].as_array().expect("findings array");
    assert_eq!(payload["summary"]["warning"].as_u64(), Some(0));
    assert!(
        findings
            .iter()
            .all(|finding| finding["severity"].as_str() != Some("warning")),
        "warning findings should be removed from json output, payload={payload}"
    );
}

#[test]
fn tm_p298_010_doctor_strict_no_warning_exits_zero_when_only_warnings_exist() {
    let fixture = setup_warning_only_fixture();
    let output = run_doctor(&fixture, "never", true, &["--strict", "--no-warning"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "strict doctor should exit 0 when only warnings are filtered out, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn tm_p298_011_doctor_strict_no_warning_exits_three_when_errors_remain() {
    let fixture = setup_error_warning_fixture();
    let output = run_doctor(&fixture, "never", true, &["--strict", "--no-warning"]);
    assert_eq!(
        output.status.code(),
        Some(3),
        "strict doctor should still exit 3 when non-warning findings remain, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn tm_p298_012_doctor_summary_table_header_reads_level() {
    let fixture = setup_level_fixture();
    let output = run_doctor(&fixture, "never", true, &[]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Level") && stdout.contains("Code") && stdout.contains("Skill"),
        "doctor summary table should use Level header, stdout={stdout}"
    );
    assert!(
        !stdout.contains("Sev"),
        "legacy Sev header must be removed, stdout={stdout}"
    );
}

#[test]
fn tm_p298_013_doctor_summary_table_uses_warning_label_instead_of_warn() {
    let fixture = setup_level_fixture();
    let output = run_doctor(&fixture, "never", true, &[]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("warning"),
        "summary table should show warning label, stdout={stdout}"
    );
    assert!(
        !stdout.lines().any(|line| line.contains(" warn ")),
        "abbreviated warn label must not appear in summary table, stdout={stdout}"
    );
}

#[test]
fn tm_p298_014_doctor_level_cell_for_error_uses_red_ansi_when_colors_enabled() {
    let fixture = setup_level_fixture();
    let output = run_doctor(&fixture, "always", true, &[]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\u{1b}[31merror"),
        "error level should render in red, stdout={stdout}"
    );
}

#[test]
fn tm_p298_015_doctor_level_cell_for_warning_uses_yellow_ansi_when_colors_enabled() {
    let fixture = setup_level_fixture();
    let output = run_doctor(&fixture, "always", true, &[]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\u{1b}[33mwarning"),
        "warning level should render in yellow, stdout={stdout}"
    );
}

#[test]
fn tm_p298_016_doctor_level_cell_for_info_uses_dim_ansi_when_colors_enabled() {
    let fixture = setup_level_fixture();
    let output = run_doctor(&fixture, "always", true, &[]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\u{1b}[2minfo"),
        "info level should render dimmed, stdout={stdout}"
    );
}

#[test]
fn tm_p298_017_doctor_level_cell_is_plain_text_with_color_never() {
    let fixture = setup_level_fixture();
    let output = run_doctor(&fixture, "never", true, &[]);
    common::assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("error") && stdout.contains("warning") && stdout.contains("info"),
        "plain-text summary should still include all level labels, stdout={stdout}"
    );
    assert!(
        !stdout.contains("\u{1b}["),
        "color-never output must not contain ANSI escapes, stdout={stdout}"
    );
}

fn setup_warning_only_fixture() -> Fixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let origin_repo = init_origin_repo(temp.path());
    let script_path = origin_repo.join("packages").join("browser").join("run.sh");
    fs::write(&script_path, "#!/bin/sh\necho hi\n").expect("write risk script");
    run_git_cmd(&origin_repo, &["add", "."]);
    run_git_cmd(&origin_repo, &["commit", "-m", "add risk script"]);

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let config_path = write_config_with_safety(
        temp.path(),
        &as_file_url(&origin_repo),
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
        true,
    );
    apply(
        config_path.to_str().expect("config path"),
        default_options(),
    )
    .expect("apply metadata-only fixture");

    Fixture {
        temp,
        home_dir,
        config_path,
        storage_root,
    }
}

fn setup_error_warning_fixture() -> Fixture {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let config_path = write_config(
        temp.path(),
        &as_file_url(&origin_repo),
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
    );

    Fixture {
        temp,
        home_dir,
        config_path,
        storage_root,
    }
}

fn setup_level_fixture() -> Fixture {
    let fixture = setup_error_warning_fixture();
    fs::create_dir_all(fixture.storage_root.join(".repos").join("orphan-cache"))
        .expect("create orphan cache entry");
    fixture
}

fn run_doctor(fixture: &Fixture, color: &str, force_tty: bool, extra_args: &[&str]) -> Output {
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
        .arg("doctor")
        .args(extra_args)
        .arg("--config")
        .arg(&fixture.config_path)
        .output()
        .expect("run doctor")
}
