mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use eden_skills_core::source::repo_cache_key;
use serde_json::Value;
use tempfile::tempdir;

use common::{
    as_file_url, assert_success, init_origin_repo, toml_escape_path, write_config, SKILL_ID,
};

#[test]
fn tm_p297_029_clean_removes_orphaned_repo_cache_entries_not_in_config() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let storage_root = temp.path().join("storage");
    let config_path = temp.path().join("skills.toml");
    let orphan_cache_dir = storage_root.join(".repos").join("github.com_old_repo@main");

    fs::create_dir_all(&orphan_cache_dir).expect("create orphan cache dir");
    fs::write(orphan_cache_dir.join("README.md"), "stale cache\n").expect("write orphan marker");
    write_empty_config(&config_path, &storage_root);

    let output = eden_command(&home_dir)
        .args(["clean", "--config"])
        .arg(&config_path)
        .output()
        .expect("run clean");

    assert_eq!(
        output.status.code(),
        Some(0),
        "clean should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !orphan_cache_dir.exists(),
        "clean should remove orphaned repo cache dir, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn tm_p297_030_clean_removes_stale_discovery_temp_dirs() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let storage_root = temp.path().join("storage");
    let config_path = temp.path().join("skills.toml");
    let stale_dir = unique_discovery_temp_dir(&temp.path().join("tmp"));

    fs::create_dir_all(&stale_dir.path).expect("create stale discovery dir");
    fs::write(stale_dir.path.join("README.md"), "stale discovery\n")
        .expect("write stale discovery marker");
    write_empty_config(&config_path, &storage_root);

    let output = eden_command(&home_dir)
        .args(["clean", "--config"])
        .arg(&config_path)
        .output()
        .expect("run clean");

    assert_success(&output);
    assert!(
        !stale_dir.path.exists(),
        "clean should remove stale discovery dir, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn tm_p297_031_clean_dry_run_lists_removals_without_deleting() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let storage_root = temp.path().join("storage");
    let config_path = temp.path().join("skills.toml");
    let orphan_cache_dir = storage_root.join(".repos").join("github.com_old_repo@main");

    fs::create_dir_all(&orphan_cache_dir).expect("create orphan cache dir");
    fs::write(orphan_cache_dir.join("README.md"), "dry run orphan\n").expect("write orphan marker");
    write_empty_config(&config_path, &storage_root);

    let output = eden_command(&home_dir)
        .args(["clean", "--dry-run", "--config"])
        .arg(&config_path)
        .output()
        .expect("run clean --dry-run");

    assert_success(&output);
    assert!(
        orphan_cache_dir.exists(),
        "dry-run should not delete orphan cache dir, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("would remove 1 orphaned cache entry"),
        "dry-run should list orphan removal, stdout={stdout}"
    );
    assert!(
        stdout.contains("Dry run complete - no files deleted"),
        "dry-run should report no deletion, stdout={stdout}"
    );
}

#[test]
fn tm_p297_032_clean_json_outputs_machine_readable_report() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let storage_root = temp.path().join("storage");
    let config_path = temp.path().join("skills.toml");
    let orphan_cache_dir = storage_root.join(".repos").join("github.com_old_repo@main");
    let stale_dir = unique_discovery_temp_dir(&temp.path().join("tmp"));
    let orphan_cache_dir_str = orphan_cache_dir.display().to_string();
    let stale_dir_str = stale_dir.path.display().to_string();

    fs::create_dir_all(&orphan_cache_dir).expect("create orphan cache dir");
    fs::write(orphan_cache_dir.join("README.md"), "json orphan\n").expect("write orphan marker");
    fs::create_dir_all(&stale_dir.path).expect("create stale discovery dir");
    fs::write(stale_dir.path.join("README.md"), "json stale discovery\n")
        .expect("write stale discovery marker");
    write_empty_config(&config_path, &storage_root);

    let output = eden_command(&home_dir)
        .args(["clean", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run clean --json");

    assert_success(&output);
    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("clean --json should emit valid json");
    assert_eq!(payload["action"], "clean");
    assert_eq!(payload["dry_run"], false);

    let removed_cache_entries = payload["removed_cache_entries"]
        .as_array()
        .expect("removed_cache_entries should be array");
    assert!(
        removed_cache_entries
            .iter()
            .any(|value| value.as_str() == Some(orphan_cache_dir_str.as_str())),
        "json payload should include orphan cache dir, payload={payload}"
    );

    let removed_discovery_dirs = payload["removed_discovery_dirs"]
        .as_array()
        .expect("removed_discovery_dirs should be array");
    assert!(
        removed_discovery_dirs
            .iter()
            .any(|value| value.as_str() == Some(stale_dir_str.as_str())),
        "json payload should include stale discovery dir, payload={payload}"
    );
    assert!(
        payload["freed_bytes"]
            .as_u64()
            .expect("freed_bytes should be u64")
            > 0,
        "clean --json should report freed bytes, payload={payload}"
    );
}

#[test]
fn tm_p297_033_clean_with_no_orphans_reports_nothing_to_clean() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let storage_root = temp.path().join("storage");
    let config_path = temp.path().join("skills.toml");

    write_empty_config(&config_path, &storage_root);

    let output = eden_command(&home_dir)
        .args(["clean", "--config"])
        .arg(&config_path)
        .output()
        .expect("run clean");

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("nothing to clean"),
        "clean should report no work, stdout={stdout}"
    );
}

#[test]
fn tm_p297_034_remove_auto_clean_runs_clean_after_removal() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let origin_repo = init_origin_repo(temp.path());
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let repo_url = as_file_url(&origin_repo);
    let config_path = write_config(
        temp.path(),
        &repo_url,
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
    );
    let orphan_cache_dir = storage_root
        .join(".repos")
        .join(repo_cache_key(&repo_url, "main"));

    fs::create_dir_all(&orphan_cache_dir).expect("create orphan cache dir");
    fs::write(orphan_cache_dir.join("README.md"), "remove auto clean\n")
        .expect("write orphan marker");

    let output = eden_command(&home_dir)
        .args(["remove", SKILL_ID, "--auto-clean", "-y", "--config"])
        .arg(&config_path)
        .output()
        .expect("run remove --auto-clean");

    assert_success(&output);
    assert!(
        !orphan_cache_dir.exists(),
        "remove --auto-clean should delete orphan cache dir, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Remove") && stdout.contains("Clean"),
        "remove --auto-clean should print both summaries, stdout={stdout}"
    );
}

#[test]
fn tm_p297_035_doctor_reports_orphan_cache_entry() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let storage_root = temp.path().join("storage");
    let config_path = temp.path().join("skills.toml");
    let orphan_cache_dir = storage_root.join(".repos").join("github.com_old_repo@main");

    fs::create_dir_all(&orphan_cache_dir).expect("create orphan cache dir");
    fs::write(orphan_cache_dir.join("README.md"), "doctor orphan\n").expect("write orphan marker");
    write_empty_config(&config_path, &storage_root);

    let output = eden_command(&home_dir)
        .args(["doctor", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run doctor --json");

    assert_success(&output);
    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("doctor --json should emit valid json");
    let findings = payload["findings"]
        .as_array()
        .expect("findings should be array");
    let orphan_finding = findings
        .iter()
        .find(|finding| finding["code"] == "ORPHAN_CACHE_ENTRY")
        .expect("expected ORPHAN_CACHE_ENTRY finding");
    assert_eq!(orphan_finding["severity"], "info");
    assert_eq!(orphan_finding["skill_id"], "");
    assert_eq!(
        orphan_finding["target_path"],
        ".repos/github.com_old_repo@main"
    );
    assert_eq!(
        orphan_finding["message"],
        "Orphaned cache entry not referenced by any configured skill"
    );
    assert_eq!(
        orphan_finding["remediation"],
        "Run `eden-skills clean` to free disk space."
    );
}

#[test]
fn tm_p297_036_clean_reports_freed_disk_space_in_human_mode() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let storage_root = temp.path().join("storage");
    let config_path = temp.path().join("skills.toml");
    let orphan_cache_dir = storage_root.join(".repos").join("github.com_old_repo@main");

    fs::create_dir_all(&orphan_cache_dir).expect("create orphan cache dir");
    fs::write(orphan_cache_dir.join("README.md"), "hello world").expect("write orphan marker");
    write_empty_config(&config_path, &storage_root);

    let output = eden_command(&home_dir)
        .args(["clean", "--config"])
        .arg(&config_path)
        .output()
        .expect("run clean");

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Freed 11 B"),
        "clean should report freed bytes, stdout={stdout}"
    );
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    let temp_root = temp_root_for_home(home_dir);
    fs::create_dir_all(&temp_root).expect("create isolated temp root");
    command.env("HOME", home_dir);
    command.env("TMPDIR", &temp_root);
    command.env("TEMP", &temp_root);
    command.env("TMP", &temp_root);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

fn write_empty_config(config_path: &Path, storage_root: &Path) {
    let contents = format!(
        "version = 1\n\n[storage]\nroot = \"{}\"\n\nskills = []\n",
        toml_escape_path(storage_root)
    );
    fs::write(config_path, contents).expect("write empty config");
}

struct DiscoveryTempGuard {
    path: PathBuf,
}

impl Drop for DiscoveryTempGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn unique_discovery_temp_dir(temp_root: &Path) -> DiscoveryTempGuard {
    fs::create_dir_all(temp_root).expect("create isolated temp root");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration since epoch")
        .as_nanos();
    let path = temp_root.join(format!(
        "eden-skills-discovery-{}-{unique}",
        std::process::id()
    ));
    DiscoveryTempGuard { path }
}

fn temp_root_for_home(home_dir: &Path) -> PathBuf {
    home_dir
        .parent()
        .expect("home dir should have a parent")
        .join("tmp")
}
