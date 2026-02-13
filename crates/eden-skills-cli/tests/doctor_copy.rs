mod common;

use std::fs;

use eden_skills_cli::commands::{apply, doctor, CommandOptions};
use eden_skills_core::config::{config_dir_from_path, load_from_file, LoadOptions};
use eden_skills_core::error::EdenError;
use eden_skills_core::plan::{build_plan, Action};
use tempfile::tempdir;

use common::{
    as_file_url, default_options, expected_source_path, init_origin_repo, write_config, SKILL_ID,
};

#[test]
fn doctor_strict_detects_missing_source() {
    let temp = tempdir().expect("tempdir");
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

    apply(
        config_path.to_str().expect("config path"),
        default_options(),
    )
    .expect("apply");
    fs::remove_dir_all(expected_source_path(&storage_root)).expect("remove source");

    let err = doctor(
        config_path.to_str().expect("config path"),
        CommandOptions {
            strict: true,
            json: false,
        },
    )
    .expect_err("doctor strict should fail");

    assert!(matches!(err, EdenError::Conflict(_)));
}

#[test]
fn copy_mode_plan_detects_source_change() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-skills");
    let config_path = write_config(
        temp.path(),
        &as_file_url(&origin_repo),
        "copy",
        &["path-exists", "content-present"],
        &storage_root,
        &target_root,
    );

    apply(
        config_path.to_str().expect("config path"),
        default_options(),
    )
    .expect("apply");

    let source_file = expected_source_path(&storage_root).join("README.txt");
    fs::write(&source_file, "v2\n").expect("update source content");

    let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
    let config_dir = config_dir_from_path(&config_path);
    let plan = build_plan(&loaded.config, &config_dir).expect("build plan");
    let item = plan
        .into_iter()
        .find(|plan_item| plan_item.skill_id == SKILL_ID)
        .expect("skill plan item");

    assert!(matches!(item.action, Action::Update));
}
