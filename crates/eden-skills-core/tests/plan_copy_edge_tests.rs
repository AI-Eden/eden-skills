use std::fs;
use std::path::Path;

use eden_skills_core::config::{
    AgentKind, Config, InstallConfig, InstallMode, SafetyConfig, SkillConfig, SourceConfig,
    TargetConfig, VerifyConfig,
};
use eden_skills_core::plan::{build_plan, Action};
use tempfile::tempdir;

fn write_bytes(path: &Path, size: usize, value: u8) {
    let buf = vec![value; size];
    fs::write(path, buf).expect("write bytes");
}

#[test]
fn copy_mode_plan_noop_streams_large_file() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("target");
    fs::create_dir_all(&storage_root).expect("create storage");
    fs::create_dir_all(&target_root).expect("create target");

    let skill_id = "copy-large";
    let source_path = storage_root.join(skill_id);
    let target_path = target_root.join(skill_id);
    fs::create_dir_all(&source_path).expect("create source");
    fs::create_dir_all(&target_path).expect("create target skill");

    // 5 MiB, large enough to avoid accidental full-file reads without being slow in CI.
    write_bytes(&source_path.join("data.bin"), 5 * 1024 * 1024, 0xA5);
    write_bytes(&target_path.join("data.bin"), 5 * 1024 * 1024, 0xA5);

    let config = Config {
        version: 1,
        storage_root: storage_root.display().to_string(),
        skills: vec![SkillConfig {
            id: skill_id.to_string(),
            source: SourceConfig {
                repo: "file:///tmp/placeholder".to_string(),
                subpath: ".".to_string(),
                r#ref: "main".to_string(),
            },
            install: InstallConfig {
                mode: InstallMode::Copy,
            },
            targets: vec![TargetConfig {
                agent: AgentKind::Custom,
                expected_path: None,
                path: Some(target_root.display().to_string()),
            }],
            verify: VerifyConfig {
                enabled: false,
                checks: vec![],
            },
            safety: SafetyConfig {
                no_exec_metadata_only: false,
            },
        }],
    };

    let plan = build_plan(&config, temp.path()).expect("build plan");
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].action, Action::Noop);
}

#[cfg(unix)]
#[test]
fn copy_mode_plan_conflict_on_unreadable_target_file() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("target");
    fs::create_dir_all(&storage_root).expect("create storage");
    fs::create_dir_all(&target_root).expect("create target");

    let skill_id = "copy-perms";
    let source_path = storage_root.join(skill_id);
    let target_path = target_root.join(skill_id);
    fs::create_dir_all(&source_path).expect("create source");
    fs::create_dir_all(&target_path).expect("create target skill");

    fs::write(source_path.join("secret.txt"), "x\n").expect("write source");
    fs::write(target_path.join("secret.txt"), "x\n").expect("write target");

    let mut perms = fs::metadata(target_path.join("secret.txt"))
        .expect("metadata")
        .permissions();
    perms.set_mode(0o000);
    fs::set_permissions(target_path.join("secret.txt"), perms).expect("set perms");

    let config = Config {
        version: 1,
        storage_root: storage_root.display().to_string(),
        skills: vec![SkillConfig {
            id: skill_id.to_string(),
            source: SourceConfig {
                repo: "file:///tmp/placeholder".to_string(),
                subpath: ".".to_string(),
                r#ref: "main".to_string(),
            },
            install: InstallConfig {
                mode: InstallMode::Copy,
            },
            targets: vec![TargetConfig {
                agent: AgentKind::Custom,
                expected_path: None,
                path: Some(target_root.display().to_string()),
            }],
            verify: VerifyConfig {
                enabled: false,
                checks: vec![],
            },
            safety: SafetyConfig {
                no_exec_metadata_only: false,
            },
        }],
    };

    let plan = build_plan(&config, temp.path()).expect("build plan");
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].action, Action::Conflict);
    assert!(
        plan[0]
            .reasons
            .iter()
            .any(|r| r.starts_with("copy comparison failed: ")),
        "expected copy comparison failure reason, got {:?}",
        plan[0].reasons
    );
}

#[cfg(unix)]
#[test]
fn copy_mode_plan_conflict_on_symlink_in_tree() {
    let temp = tempdir().expect("tempdir");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("target");
    fs::create_dir_all(&storage_root).expect("create storage");
    fs::create_dir_all(&target_root).expect("create target");

    let skill_id = "copy-symlink";
    let source_path = storage_root.join(skill_id);
    let target_path = target_root.join(skill_id);
    fs::create_dir_all(&source_path).expect("create source");
    fs::create_dir_all(&target_path).expect("create target skill");

    fs::write(source_path.join("a.txt"), "x\n").expect("write source");
    fs::write(target_path.join("a.txt"), "x\n").expect("write target");

    #[cfg(unix)]
    std::os::unix::fs::symlink(source_path.join("a.txt"), source_path.join("link.txt"))
        .expect("create symlink");

    let config = Config {
        version: 1,
        storage_root: storage_root.display().to_string(),
        skills: vec![SkillConfig {
            id: skill_id.to_string(),
            source: SourceConfig {
                repo: "file:///tmp/placeholder".to_string(),
                subpath: ".".to_string(),
                r#ref: "main".to_string(),
            },
            install: InstallConfig {
                mode: InstallMode::Copy,
            },
            targets: vec![TargetConfig {
                agent: AgentKind::Custom,
                expected_path: None,
                path: Some(target_root.display().to_string()),
            }],
            verify: VerifyConfig {
                enabled: false,
                checks: vec![],
            },
            safety: SafetyConfig {
                no_exec_metadata_only: false,
            },
        }],
    };

    let plan = build_plan(&config, temp.path()).expect("build plan");
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].action, Action::Conflict);
    assert!(
        plan[0]
            .reasons
            .iter()
            .any(|r| r == "copy comparison failed: symlink in tree"),
        "expected symlink-in-tree conflict, got {:?}",
        plan[0].reasons
    );
}
