mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

use common::{as_file_url, init_origin_repo, write_config};

#[test]
fn tm_p297_047_error_hints_use_tilde_arrow_prefix_instead_of_unicode_arrow() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");
    let missing_config = temp.path().join("does-not-exist").join("skills.toml");

    let output = eden_command(&home_dir)
        .args(["list", "--color", "never", "--config"])
        .arg(&missing_config)
        .output()
        .expect("run list with missing config");

    assert_eq!(
        output.status.code(),
        Some(1),
        "missing config should fail with runtime exit code 1, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("  ~> ") && !stderr.contains("  → "),
        "error hints must use ~> prefix and not the unicode arrow, stderr={stderr}"
    );
}

#[test]
fn tm_p297_048_error_hints_render_tilde_arrow_prefix_in_magenta_when_colors_enabled() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");
    let missing_config = temp.path().join("does-not-exist").join("skills.toml");

    let output = eden_command(&home_dir)
        .args(["list", "--color", "always", "--config"])
        .arg(&missing_config)
        .output()
        .expect("run list with missing config");

    assert_eq!(
        output.status.code(),
        Some(1),
        "missing config should fail with runtime exit code 1, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("\u{1b}[35m~>"),
        "colored error hints must style the ~> prefix in magenta, stderr={stderr}"
    );
}

#[test]
fn tm_p297_049_doctor_remediation_uses_magenta_tilde_arrow_prefix() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");

    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config(
        temp.path(),
        &as_file_url(&origin_repo),
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
    );

    let output = eden_command(&home_dir)
        .env("EDEN_SKILLS_FORCE_TTY", "1")
        .args(["doctor", "--color", "always", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor");

    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor should succeed without --strict, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\u{1b}[35m~>"),
        "doctor remediation lines must style the ~> prefix in magenta, stdout={stdout}"
    );
}

#[test]
fn tm_p297_050_update_guidance_uses_tilde_arrow_prefix() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    fs::create_dir_all(&home_dir).expect("create HOME");
    let config_path = temp.path().join("skills.toml");
    let storage_root = temp.path().join("storage");
    fs::write(
        &config_path,
        format!(
            "version = 1\n\n[storage]\nroot = \"{}\"\n\nskills = []\n",
            toml_escape_path(&storage_root)
        ),
    )
    .expect("write empty config");

    let output = eden_command(&home_dir)
        .args(["update", "--color", "never", "--config"])
        .arg(&config_path)
        .output()
        .expect("run update");

    assert_eq!(
        output.status.code(),
        Some(0),
        "empty-state update should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("  ~> Run 'eden-skills install <owner/repo>' to get started.")
            && !stdout.contains("  → "),
        "update guidance must use the ~> prefix and not the unicode arrow, stdout={stdout}"
    );
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}
