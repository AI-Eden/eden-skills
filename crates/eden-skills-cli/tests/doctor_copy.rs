mod common;

use std::fs;

use eden_skills_cli::commands::{apply, doctor, CommandOptions};
use eden_skills_core::config::{config_dir_from_path, load_from_file, LoadOptions};
use eden_skills_core::error::EdenError;
use eden_skills_core::plan::{build_plan, Action};
use tempfile::tempdir;

use common::{
    as_file_url, default_options, expected_source_path, init_origin_repo, run_git_cmd,
    write_config, SKILL_ID,
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
        false,
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
    fs::write(&source_file, "v2-updated\n").expect("update source content");

    let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
    let config_dir = config_dir_from_path(&config_path);
    let plan = build_plan(&loaded.config, &config_dir).expect("build plan");
    let item = plan
        .into_iter()
        .find(|plan_item| plan_item.skill_id == SKILL_ID)
        .expect("skill plan item");

    assert!(matches!(item.action, Action::Update));
}

#[test]
fn tm_p295_036_doctor_resolves_remote_sources_from_repo_cache() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());
    fs::write(
        origin_repo.join("LICENSE"),
        "MIT License\n\nPermission is hereby granted, free of charge, to any person obtaining a copy...\n",
    )
    .expect("write license");
    run_git_cmd(&origin_repo, &["add", "."]);
    run_git_cmd(&origin_repo, &["commit", "-m", "add license"]);

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

    let repo_cache_source = expected_source_path(&storage_root);
    let legacy_source = storage_root.join(SKILL_ID).join("packages").join("browser");
    assert!(
        repo_cache_source.starts_with(storage_root.join(".repos")),
        "expected resolved source to live under repo cache, source={}",
        repo_cache_source.display()
    );
    assert!(
        !legacy_source.exists(),
        "legacy per-skill source path should be absent for remote installs"
    );

    doctor(
        config_path.to_str().expect("config path"),
        CommandOptions {
            strict: true,
            json: false,
        },
        false,
    )
    .expect("doctor should accept repo-cache-backed source paths");
}
