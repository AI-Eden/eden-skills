#[cfg(windows)]
mod windows {
    use std::fs;
    use std::path::{Path, PathBuf};

    use eden_skills_core::adapter::{LocalAdapter, TargetAdapter};
    use eden_skills_core::config::{
        AgentKind, Config, InstallConfig, InstallMode, ReactorConfig, SafetyConfig, SkillConfig,
        SourceConfig, TargetConfig, VerifyConfig,
    };
    use eden_skills_core::plan::{build_plan, Action};
    use eden_skills_core::source::resolve_skill_source_path;
    use tempfile::tempdir;

    #[test]
    fn plan_treats_matching_junction_as_noop_for_symlink_mode() {
        let temp = tempdir().expect("tempdir");
        let storage_root = temp.path().join("storage");
        let target_root = temp.path().join("target");
        let skill_id = "tm-p295-020";
        let config = symlink_mode_config(&storage_root, &target_root, skill_id);
        let source_path = resolve_skill_source_path(&storage_root, &config.skills[0]);
        let target_path = target_root.join(skill_id);

        fs::create_dir_all(&source_path).expect("create source");
        fs::create_dir_all(&target_root).expect("create target root");
        fs::write(source_path.join("README.md"), "demo\n").expect("write source file");
        junction::create(&source_path, &target_path).expect("create junction");
        assert!(
            junction::exists(&target_path).expect("junction exists"),
            "test precondition: target should be a junction"
        );

        let plan = build_plan(&config, temp.path()).expect("build plan");
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan[0].action,
            Action::Noop,
            "junction-backed target should be treated like a valid symlink target"
        );
    }

    #[tokio::test]
    async fn local_adapter_reinstall_replaces_existing_junction_without_touching_source() {
        let temp = tempdir().expect("tempdir");
        let source_a = temp.path().join("source-a");
        let source_b = temp.path().join("source-b");
        let target = temp.path().join("installed-skill");

        fs::create_dir_all(&source_a).expect("create source a");
        fs::create_dir_all(&source_b).expect("create source b");
        fs::write(source_a.join("keep.txt"), "keep\n").expect("write source a file");
        fs::write(source_b.join("README.md"), "new\n").expect("write source b file");

        junction::create(&source_a, &target).expect("create initial junction");
        assert!(
            junction::exists(&target).expect("junction exists"),
            "test precondition: target should start as a junction"
        );

        let adapter = LocalAdapter::new();
        adapter
            .install(&source_b, &target, InstallMode::Symlink)
            .await
            .expect("reinstall over existing junction");

        assert!(
            source_a.join("keep.txt").exists(),
            "reinstall should remove the junction, not the source directory contents"
        );
        assert_eq!(
            resolved_reparse_target(&target),
            fs::canonicalize(&source_b).expect("canonicalize source b"),
            "target should resolve to the new source after reinstall"
        );
    }

    #[test]
    fn junction_crate_round_trip_smoke_test() {
        let temp = tempdir().expect("tempdir");
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(&source).expect("create source");
        fs::write(source.join("README.md"), "demo\n").expect("write source file");

        junction::create(&source, &target).expect("create junction");
        assert!(
            junction::exists(&target).expect("junction exists"),
            "junction crate should be usable from Windows builds"
        );

        junction::delete(&target).expect("delete junction");
        assert!(
            target.exists(),
            "junction::delete removes the reparse point but leaves the directory path behind"
        );
        assert!(
            !junction::exists(&target).unwrap_or(false),
            "junction path should no longer report as a junction after delete"
        );
        assert!(
            !target.join("README.md").exists(),
            "junction path should no longer resolve into the original target contents after delete"
        );
    }

    fn symlink_mode_config(storage_root: &Path, target_root: &Path, skill_id: &str) -> Config {
        Config {
            version: 1,
            storage_root: storage_root.display().to_string(),
            reactor: ReactorConfig::default(),
            skills: vec![SkillConfig {
                id: skill_id.to_string(),
                source: SourceConfig {
                    repo: "file:///tmp/placeholder".to_string(),
                    subpath: ".".to_string(),
                    r#ref: "main".to_string(),
                },
                install: InstallConfig {
                    mode: InstallMode::Symlink,
                },
                targets: vec![TargetConfig {
                    agent: AgentKind::Custom,
                    expected_path: None,
                    path: Some(target_root.display().to_string()),
                    environment: "local".to_string(),
                }],
                verify: VerifyConfig {
                    enabled: false,
                    checks: vec![],
                },
                safety: SafetyConfig {
                    no_exec_metadata_only: false,
                },
            }],
        }
    }

    fn resolved_reparse_target(path: &Path) -> PathBuf {
        let raw = if junction::exists(path).unwrap_or(false) {
            junction::get_target(path).expect("read junction target")
        } else {
            fs::read_link(path).expect("read symlink target")
        };
        let resolved = if raw.is_absolute() {
            raw
        } else {
            path.parent().unwrap_or(Path::new(".")).join(raw)
        };
        fs::canonicalize(resolved).expect("canonicalize target")
    }
}

#[cfg(not(windows))]
#[test]
fn junction_support_is_cfg_gated_on_non_windows() {
    assert_eq!(
        eden_skills_core::config::InstallMode::Symlink.as_str(),
        "symlink"
    );
}
