mod common;

use std::fs;
use std::path::Path;

use eden_skills_cli::commands::{apply, repair};
use eden_skills_core::error::EdenError;
use tempfile::tempdir;

use common::{
    as_file_url, create_symlink, default_options, expected_source_path, expected_target_path,
    init_origin_repo, make_read_only_dir, resolved_symlink, restore_permissions, write_config,
};

#[test]
fn fresh_and_repeated_apply_symlink() {
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
    .expect("first apply");
    let target = expected_target_path(&target_root);
    assert!(fs::symlink_metadata(&target)
        .expect("target metadata")
        .file_type()
        .is_symlink());
    assert_eq!(
        resolved_symlink(&target),
        expected_source_path(&storage_root)
    );

    apply(
        config_path.to_str().expect("config path"),
        default_options(),
    )
    .expect("second apply");
    assert_eq!(
        resolved_symlink(&target),
        expected_source_path(&storage_root)
    );
}

#[test]
fn repair_recovers_broken_symlink() {
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
    fs::remove_file(&target).expect("remove existing symlink");
    create_symlink(Path::new("/tmp/eden-skills-broken"), &target).expect("broken symlink");

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

#[cfg(unix)]
#[test]
fn apply_fails_on_permission_denied_target_path() {
    let temp = tempdir().expect("tempdir");
    let origin_repo = init_origin_repo(temp.path());

    let storage_root = temp.path().join("storage");
    let restricted_parent = temp.path().join("restricted");
    let original_permissions = make_read_only_dir(&restricted_parent);

    let target_root = restricted_parent.join("agent-skills");
    let config_path = write_config(
        temp.path(),
        &as_file_url(&origin_repo),
        "symlink",
        &["path-exists", "target-resolves", "is-symlink"],
        &storage_root,
        &target_root,
    );

    let result = apply(
        config_path.to_str().expect("config path"),
        default_options(),
    );

    restore_permissions(&restricted_parent, original_permissions);

    let err = result.expect_err("apply should fail for permission denied target");
    assert!(
        matches!(err, EdenError::Io(_) | EdenError::Runtime(_)),
        "unexpected error variant: {err}"
    );
}
