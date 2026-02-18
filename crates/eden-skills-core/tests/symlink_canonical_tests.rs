#[cfg(unix)]
mod unix {
    use std::fs;
    use std::os::unix::fs::symlink;

    use eden_skills_core::config::{config_dir_from_path, load_from_file, LoadOptions};
    use eden_skills_core::plan::{build_plan, Action};
    use eden_skills_core::verify::verify_config_state;
    use tempfile::tempdir;

    #[test]
    fn verify_target_resolves_uses_canonical_paths() {
        let temp = tempdir().expect("tempdir");
        let storage_root = temp.path().join("storage");
        let target_root = temp.path().join("agent-skills");
        let config_path = temp.path().join("skills.toml");

        let skill_id = "x";
        let source_subpath = "src";
        let source_real = storage_root.join(skill_id).join(source_subpath);
        fs::create_dir_all(&source_real).expect("create source dir");
        fs::write(source_real.join("file.txt"), "ok\n").expect("write source file");

        let alias = temp.path().join("alias-src");
        symlink(&source_real, &alias).expect("create alias symlink to source");

        fs::create_dir_all(&target_root).expect("create target root");
        let target_link = target_root.join(skill_id);
        symlink(&alias, &target_link).expect("create target symlink to alias");

        fs::write(
            &config_path,
            format!(
                "version = 1\n\n[storage]\nroot = \"{}\"\n\n[[skills]]\nid = \"{}\"\n\n[skills.source]\nrepo = \"file:///tmp/unused\"\nsubpath = \"{}\"\nref = \"main\"\n\n[skills.install]\nmode = \"symlink\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{}\"\n\n[skills.verify]\nenabled = true\nchecks = [\"path-exists\", \"is-symlink\", \"target-resolves\"]\n",
                storage_root.display(),
                skill_id,
                source_subpath,
                target_root.display()
            ),
        )
        .expect("write config");

        let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
        let config_dir = config_dir_from_path(&config_path);
        let issues = verify_config_state(&loaded.config, &config_dir).expect("verify state");
        assert!(
            issues.is_empty(),
            "expected no verify issues, got: {issues:?}"
        );
    }

    #[test]
    fn plan_symlink_noop_uses_canonical_paths() {
        let temp = tempdir().expect("tempdir");
        let storage_root = temp.path().join("storage");
        let target_root = temp.path().join("agent-skills");
        let config_path = temp.path().join("skills.toml");

        let skill_id = "x";
        let source_subpath = "src";
        let source_real = storage_root.join(skill_id).join(source_subpath);
        fs::create_dir_all(&source_real).expect("create source dir");
        fs::write(source_real.join("file.txt"), "ok\n").expect("write source file");

        let alias = temp.path().join("alias-src");
        symlink(&source_real, &alias).expect("create alias symlink to source");

        fs::create_dir_all(&target_root).expect("create target root");
        let target_link = target_root.join(skill_id);
        symlink(&alias, &target_link).expect("create target symlink to alias");

        fs::write(
            &config_path,
            format!(
                "version = 1\n\n[storage]\nroot = \"{}\"\n\n[[skills]]\nid = \"{}\"\n\n[skills.source]\nrepo = \"file:///tmp/unused\"\nsubpath = \"{}\"\nref = \"main\"\n\n[skills.install]\nmode = \"symlink\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{}\"\n",
                storage_root.display(),
                skill_id,
                source_subpath,
                target_root.display()
            ),
        )
        .expect("write config");

        let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
        let config_dir = config_dir_from_path(&config_path);
        let plan = build_plan(&loaded.config, &config_dir).expect("build plan");
        let item = plan
            .iter()
            .find(|it| it.skill_id == skill_id)
            .expect("plan item");
        assert!(
            matches!(item.action, Action::Noop),
            "expected noop action, got: {:?} reasons={:?}",
            item.action,
            item.reasons
        );
    }
}

#[cfg(windows)]
mod windows {
    use std::fs;
    use std::os::windows::fs::symlink_dir;

    use eden_skills_core::config::{config_dir_from_path, load_from_file, LoadOptions};
    use eden_skills_core::plan::{build_plan, Action};
    use eden_skills_core::verify::verify_config_state;
    use tempfile::tempdir;

    #[test]
    fn verify_target_resolves_uses_canonical_paths() {
        let temp = tempdir().expect("tempdir");
        let storage_root = temp.path().join("storage");
        let target_root = temp.path().join("agent-skills");
        let config_path = temp.path().join("skills.toml");

        let skill_id = "x";
        let source_subpath = "src";
        let source_real = storage_root.join(skill_id).join(source_subpath);
        fs::create_dir_all(&source_real).expect("create source dir");
        fs::write(source_real.join("file.txt"), "ok\n").expect("write source file");

        let alias = temp.path().join("alias-src");
        symlink_dir(&source_real, &alias).expect("create alias symlink to source");

        fs::create_dir_all(&target_root).expect("create target root");
        let target_link = target_root.join(skill_id);
        symlink_dir(&alias, &target_link).expect("create target symlink to alias");

        fs::write(
            &config_path,
            format!(
                "version = 1\n\n[storage]\nroot = \"{}\"\n\n[[skills]]\nid = \"{}\"\n\n[skills.source]\nrepo = \"file:///tmp/unused\"\nsubpath = \"{}\"\nref = \"main\"\n\n[skills.install]\nmode = \"symlink\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{}\"\n\n[skills.verify]\nenabled = true\nchecks = [\"path-exists\", \"is-symlink\", \"target-resolves\"]\n",
                storage_root.display(),
                skill_id,
                source_subpath,
                target_root.display()
            ),
        )
        .expect("write config");

        let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
        let config_dir = config_dir_from_path(&config_path);
        let issues = verify_config_state(&loaded.config, &config_dir).expect("verify state");
        assert!(
            issues.is_empty(),
            "expected no verify issues, got: {issues:?}"
        );
    }

    #[test]
    fn plan_symlink_noop_uses_canonical_paths() {
        let temp = tempdir().expect("tempdir");
        let storage_root = temp.path().join("storage");
        let target_root = temp.path().join("agent-skills");
        let config_path = temp.path().join("skills.toml");

        let skill_id = "x";
        let source_subpath = "src";
        let source_real = storage_root.join(skill_id).join(source_subpath);
        fs::create_dir_all(&source_real).expect("create source dir");
        fs::write(source_real.join("file.txt"), "ok\n").expect("write source file");

        let alias = temp.path().join("alias-src");
        symlink_dir(&source_real, &alias).expect("create alias symlink to source");

        fs::create_dir_all(&target_root).expect("create target root");
        let target_link = target_root.join(skill_id);
        symlink_dir(&alias, &target_link).expect("create target symlink to alias");

        fs::write(
            &config_path,
            format!(
                "version = 1\n\n[storage]\nroot = \"{}\"\n\n[[skills]]\nid = \"{}\"\n\n[skills.source]\nrepo = \"file:///tmp/unused\"\nsubpath = \"{}\"\nref = \"main\"\n\n[skills.install]\nmode = \"symlink\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{}\"\n",
                storage_root.display(),
                skill_id,
                source_subpath,
                target_root.display()
            ),
        )
        .expect("write config");

        let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
        let config_dir = config_dir_from_path(&config_path);
        let plan = build_plan(&loaded.config, &config_dir).expect("build plan");
        let item = plan
            .iter()
            .find(|it| it.skill_id == skill_id)
            .expect("plan item");
        assert!(
            matches!(item.action, Action::Noop),
            "expected noop action, got: {:?} reasons={:?}",
            item.action,
            item.reasons
        );
    }
}
