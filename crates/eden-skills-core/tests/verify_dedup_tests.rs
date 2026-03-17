use std::fs;
use std::path::Path;

use eden_skills_core::config::{config_dir_from_path, load_from_file, LoadOptions};
use eden_skills_core::source::resolve_repo_cache_root;
use eden_skills_core::verify::{verify_config_state, VerifyIssue};
use tempfile::tempdir;

const SKILL_ID: &str = "dedup-skill";
const SOURCE_REPO: &str = "https://example.com/demo.git";
const SOURCE_SUBPATH: &str = "packages/browser";

#[test]
fn tm_p298_018_missing_target_reports_only_path_exists_issue() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = temp.path().join("skills.toml");

    write_verify_config(
        &config_path,
        &storage_root,
        &target_root,
        &["path-exists", "is-symlink", "target-resolves"],
    );

    let issues = load_verify_issues(&config_path);

    assert_eq!(
        issues.len(),
        1,
        "missing targets should collapse to one issue, got: {issues:?}"
    );
    assert_eq!(issues[0].check, "path-exists");
    assert_eq!(issues[0].message, "target path does not exist");
}

#[test]
fn tm_p298_019_existing_wrong_symlink_still_reports_target_resolve_mismatch() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = temp.path().join("skills.toml");

    let expected_source =
        resolve_repo_cache_root(&storage_root, SOURCE_REPO, "main").join(SOURCE_SUBPATH);
    fs::create_dir_all(&expected_source).expect("create expected source dir");
    fs::write(expected_source.join("README.md"), "expected\n").expect("write expected source file");

    let wrong_source = temp.path().join("wrong-source");
    fs::create_dir_all(&wrong_source).expect("create wrong source dir");
    fs::write(wrong_source.join("README.md"), "wrong\n").expect("write wrong source file");

    fs::create_dir_all(&target_root).expect("create target root");
    create_dir_symlink(&wrong_source, &target_root.join(SKILL_ID)).expect("create wrong symlink");

    write_verify_config(
        &config_path,
        &storage_root,
        &target_root,
        &["path-exists", "is-symlink", "target-resolves"],
    );

    let issues = load_verify_issues(&config_path);

    assert_eq!(
        issues.len(),
        1,
        "existing wrong symlink should still report one mismatch, got: {issues:?}"
    );
    assert_eq!(issues[0].check, "target-resolves");
    assert!(
        issues[0].message.contains("resolves to"),
        "expected target-resolves mismatch message, got: {:?}",
        issues[0]
    );
}

fn load_verify_issues(config_path: &Path) -> Vec<VerifyIssue> {
    let loaded = load_from_file(config_path, LoadOptions::default()).expect("load config");
    let config_dir = config_dir_from_path(config_path);
    verify_config_state(&loaded.config, &config_dir).expect("verify config state")
}

fn write_verify_config(
    config_path: &Path,
    storage_root: &Path,
    target_root: &Path,
    checks: &[&str],
) {
    let checks = checks
        .iter()
        .map(|check| format!("\"{check}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let config = format!(
        "version = 1\n\n[storage]\nroot = \"{storage_root}\"\n\n[[skills]]\nid = \"{skill_id}\"\n\n[skills.source]\nrepo = \"{repo}\"\nsubpath = \"{subpath}\"\nref = \"main\"\n\n[skills.install]\nmode = \"symlink\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{target_root}\"\n\n[skills.verify]\nenabled = true\nchecks = [{checks}]\n",
        storage_root = toml_escape_path(storage_root),
        skill_id = SKILL_ID,
        repo = SOURCE_REPO,
        subpath = SOURCE_SUBPATH,
        target_root = toml_escape_path(target_root),
    );
    fs::write(config_path, config).expect("write config");
}

fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

#[cfg(unix)]
fn create_dir_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
fn create_dir_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(source, target)
}
