mod common;

use eden_skills_cli::commands::{apply, repair};
use eden_skills_core::config::{config_dir_from_path, load_from_file, LoadOptions};
use eden_skills_core::verify::verify_config_state;
use tempfile::tempdir;

use common::{
    as_file_url, default_options, expected_source_path, expected_target_path, init_origin_repo,
    remove_symlink, resolved_symlink, write_config,
};

#[test]
fn tm_p298_020_repair_restores_missing_symlink_after_single_verify_finding() {
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

    let target = expected_target_path(&target_root);
    remove_symlink(&target).expect("remove installed symlink");

    let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
    let config_dir = config_dir_from_path(&config_path);
    let issues = verify_config_state(&loaded.config, &config_dir).expect("verify config state");
    assert_eq!(
        issues.len(),
        1,
        "missing target should produce a single verify issue before repair, got: {issues:?}"
    );
    assert_eq!(issues[0].check, "path-exists");

    repair(
        config_path.to_str().expect("config path"),
        default_options(),
    )
    .expect("repair");

    assert_eq!(
        resolved_symlink(&target),
        expected_source_path(&storage_root)
    );
}
