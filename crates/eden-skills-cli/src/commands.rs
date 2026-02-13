use std::fs;
use std::path::{Path, PathBuf};

use eden_skills_core::config::InstallMode;
use eden_skills_core::config::{config_dir_from_path, load_from_file, LoadOptions};
use eden_skills_core::error::EdenError;
use eden_skills_core::plan::{build_plan, Action, PlanItem};
use eden_skills_core::source::sync_sources;
use eden_skills_core::verify::verify_config_state;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommandOptions {
    pub strict: bool,
    pub json: bool,
}

pub fn plan(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path = Path::new(config_path);
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }

    let config_dir = config_dir_from_path(config_path);
    let plan = build_plan(&loaded.config, &config_dir)?;
    if options.json {
        print_plan_json(&plan)?;
    } else {
        print_plan_text(&plan);
    }
    Ok(())
}

pub fn apply(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path = Path::new(config_path);
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    let config_dir = config_dir_from_path(config_path);
    let sync_summary = sync_sources(&loaded.config, &config_dir)?;
    println!(
        "source sync: cloned={} updated={} skipped={}",
        sync_summary.cloned, sync_summary.updated, sync_summary.skipped
    );
    let plan = build_plan(&loaded.config, &config_dir)?;

    let mut created = 0usize;
    let mut updated = 0usize;
    let mut noops = 0usize;
    let mut conflicts = 0usize;

    for item in &plan {
        match item.action {
            Action::Create => {
                apply_plan_item(item)?;
                created += 1;
            }
            Action::Update => {
                apply_plan_item(item)?;
                updated += 1;
            }
            Action::Noop => {
                noops += 1;
            }
            Action::Conflict => {
                conflicts += 1;
            }
        }
    }

    println!("apply summary: create={created} update={updated} noop={noops} conflict={conflicts}");

    if options.strict && conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "strict mode blocked apply: {conflicts} conflict entries"
        )));
    }

    let verify_issues = verify_config_state(&loaded.config, &config_dir)?;
    if !verify_issues.is_empty() {
        return Err(EdenError::Runtime(format!(
            "post-apply verification failed with {} issue(s); first: [{}] {} {}",
            verify_issues.len(),
            verify_issues[0].check,
            verify_issues[0].skill_id,
            verify_issues[0].message
        )));
    }

    println!("apply verification: ok");
    Ok(())
}

pub fn doctor(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path = Path::new(config_path);
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    let config_dir = config_dir_from_path(config_path);
    let plan = build_plan(&loaded.config, &config_dir)?;
    let verify_issues = verify_config_state(&loaded.config, &config_dir)?;

    let mut findings = Vec::new();
    for item in &plan {
        if matches!(item.action, Action::Conflict) {
            findings.push(format!(
                "CONFLICT {} {} ({})",
                item.skill_id,
                item.target_path,
                item.reasons.join("; ")
            ));
        }
    }
    for issue in &verify_issues {
        findings.push(format!(
            "VERIFY {} {} [{}] {}",
            issue.skill_id, issue.target_path, issue.check, issue.message
        ));
    }

    if findings.is_empty() {
        println!("doctor: no issues detected");
        return Ok(());
    }

    println!("doctor: detected {} issue(s)", findings.len());
    for line in &findings {
        println!("  {line}");
    }

    if options.strict {
        return Err(EdenError::Conflict(format!(
            "doctor found {} issue(s) in strict mode",
            findings.len()
        )));
    }
    Ok(())
}

pub fn repair(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path = Path::new(config_path);
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    let config_dir = config_dir_from_path(config_path);
    let sync_summary = sync_sources(&loaded.config, &config_dir)?;
    println!(
        "source sync: cloned={} updated={} skipped={}",
        sync_summary.cloned, sync_summary.updated, sync_summary.skipped
    );
    let plan = build_plan(&loaded.config, &config_dir)?;

    let mut repaired = 0usize;
    let mut skipped_conflicts = 0usize;

    for item in &plan {
        match item.action {
            Action::Create | Action::Update => {
                apply_plan_item(item)?;
                repaired += 1;
            }
            Action::Conflict => {
                skipped_conflicts += 1;
            }
            Action::Noop => {}
        }
    }

    println!("repair summary: repaired={repaired} skipped_conflicts={skipped_conflicts}");

    let verify_issues = verify_config_state(&loaded.config, &config_dir)?;
    if !verify_issues.is_empty() {
        return Err(EdenError::Runtime(format!(
            "post-repair verification failed with {} issue(s); first: [{}] {} {}",
            verify_issues.len(),
            verify_issues[0].check,
            verify_issues[0].skill_id,
            verify_issues[0].message
        )));
    }

    if options.strict && skipped_conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "repair skipped {skipped_conflicts} conflict entries in strict mode"
        )));
    }

    println!("repair verification: ok");
    Ok(())
}

fn print_plan_text(items: &[PlanItem]) {
    for item in items {
        println!(
            "{} {} {} -> {} ({})",
            action_label(item.action),
            item.skill_id,
            item.source_path,
            item.target_path,
            item.install_mode.as_str()
        );
        for reason in &item.reasons {
            println!("  reason: {reason}");
        }
    }
}

fn print_plan_json(items: &[PlanItem]) -> Result<(), EdenError> {
    let payload = serde_json::to_string_pretty(items)
        .map_err(|err| EdenError::Runtime(format!("failed to serialize plan as json: {err}")))?;
    println!("{payload}");
    Ok(())
}

fn action_label(action: Action) -> &'static str {
    match action {
        Action::Create => "create",
        Action::Update => "update",
        Action::Noop => "noop",
        Action::Conflict => "conflict",
    }
}

fn apply_plan_item(item: &PlanItem) -> Result<(), EdenError> {
    let source_path = PathBuf::from(&item.source_path);
    let target_path = PathBuf::from(&item.target_path);

    if !source_path.exists() {
        return Err(EdenError::Runtime(format!(
            "source path missing for skill `{}`: {}",
            item.skill_id, item.source_path
        )));
    }

    match item.install_mode {
        InstallMode::Symlink => apply_symlink(&source_path, &target_path),
        InstallMode::Copy => apply_copy(&source_path, &target_path),
    }
}

fn apply_symlink(source_path: &Path, target_path: &Path) -> Result<(), EdenError> {
    ensure_parent_dir(target_path)?;
    if fs::symlink_metadata(target_path).is_ok() {
        remove_path(target_path)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source_path, target_path)?;
    }
    #[cfg(windows)]
    {
        if source_path.is_dir() {
            std::os::windows::fs::symlink_dir(source_path, target_path)?;
        } else {
            std::os::windows::fs::symlink_file(source_path, target_path)?;
        }
    }

    Ok(())
}

fn apply_copy(source_path: &Path, target_path: &Path) -> Result<(), EdenError> {
    ensure_parent_dir(target_path)?;
    if fs::symlink_metadata(target_path).is_ok() {
        remove_path(target_path)?;
    }
    copy_recursively(source_path, target_path)
}

fn ensure_parent_dir(path: &Path) -> Result<(), EdenError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn remove_path(path: &Path) -> Result<(), EdenError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path)?;
        return Ok(());
    }
    if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn copy_recursively(source: &Path, target: &Path) -> Result<(), EdenError> {
    if source.is_file() {
        fs::copy(source, target)?;
        return Ok(());
    }

    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let child_source = entry.path();
        let child_target = target.join(entry.file_name());
        if child_source.is_dir() {
            copy_recursively(&child_source, &child_target)?;
        } else {
            fs::copy(&child_source, &child_target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use eden_skills_core::config::{
        config_dir_from_path, load_from_file, InstallMode, LoadOptions,
    };
    use eden_skills_core::error::EdenError;
    use eden_skills_core::plan::{build_plan, Action, PlanItem};
    use serde_json::json;
    use tempfile::tempdir;

    use super::{apply, doctor, repair, CommandOptions};

    const SKILL_ID: &str = "demo-skill";

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

    #[test]
    fn plan_json_serialization_keeps_stable_fields() {
        let item = PlanItem {
            skill_id: "demo-skill".to_string(),
            source_path: "/tmp/source".to_string(),
            target_path: "/tmp/target".to_string(),
            install_mode: InstallMode::Symlink,
            action: Action::Create,
            reasons: vec!["target path does not exist".to_string()],
        };

        let payload = serde_json::to_value(vec![item]).expect("serialize plan json");
        let first = payload[0].as_object().expect("plan entry object");

        assert_eq!(first.get("skill_id"), Some(&json!("demo-skill")));
        assert_eq!(first.get("install_mode"), Some(&json!("symlink")));
        assert_eq!(first.get("action"), Some(&json!("create")));
        assert_eq!(
            first.get("reasons"),
            Some(&json!(["target path does not exist"]))
        );
    }

    #[cfg(unix)]
    #[test]
    fn apply_fails_on_permission_denied_target_path() {
        let temp = tempdir().expect("tempdir");
        let origin_repo = init_origin_repo(temp.path());

        let storage_root = temp.path().join("storage");
        let restricted_parent = temp.path().join("restricted");
        fs::create_dir_all(&restricted_parent).expect("create restricted parent");
        let original_permissions = fs::metadata(&restricted_parent)
            .expect("restricted metadata")
            .permissions();

        let mut read_exec_only = original_permissions.clone();
        read_exec_only.set_mode(0o555);
        fs::set_permissions(&restricted_parent, read_exec_only)
            .expect("set restricted parent permissions");

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

        fs::set_permissions(&restricted_parent, original_permissions)
            .expect("restore restricted parent permissions");

        let err = result.expect_err("apply should fail for permission denied target");
        assert!(
            matches!(err, EdenError::Io(_) | EdenError::Runtime(_)),
            "unexpected error variant: {err}"
        );
    }

    fn default_options() -> CommandOptions {
        CommandOptions {
            strict: false,
            json: false,
        }
    }

    fn init_origin_repo(base: &Path) -> PathBuf {
        let repo = base.join("origin-repo");
        fs::create_dir_all(repo.join("packages").join("browser")).expect("create repo tree");
        fs::write(
            repo.join("packages").join("browser").join("README.txt"),
            "v1\n",
        )
        .expect("write seed file");

        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "test@example.com"]);
        run_git(&repo, &["config", "user.name", "eden-skills-test"]);
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "init"]);
        run_git(&repo, &["branch", "-M", "main"]);
        repo
    }

    fn run_git(cwd: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("spawn git");
        if output.status.success() {
            return;
        }

        panic!(
            "git {:?} failed in {}: status={} stderr=`{}` stdout=`{}`",
            args,
            cwd.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim(),
            String::from_utf8_lossy(&output.stdout).trim()
        );
    }

    fn write_config(
        base: &Path,
        repo_url: &str,
        install_mode: &str,
        verify_checks: &[&str],
        storage_root: &Path,
        target_root: &Path,
    ) -> PathBuf {
        let checks = verify_checks
            .iter()
            .map(|check| format!("\"{check}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let config = format!(
            "version = 1\n\n[storage]\nroot = \"{}\"\n\n[[skills]]\nid = \"{}\"\n\n[skills.source]\nrepo = \"{}\"\nsubpath = \"packages/browser\"\nref = \"main\"\n\n[skills.install]\nmode = \"{}\"\n\n[[skills.targets]]\nagent = \"custom\"\npath = \"{}\"\n\n[skills.verify]\nenabled = true\nchecks = [{}]\n\n[skills.safety]\nno_exec_metadata_only = false\n",
            toml_escape(storage_root),
            SKILL_ID,
            toml_escape_str(repo_url),
            install_mode,
            toml_escape(target_root),
            checks
        );
        let config_path = base.join("skills.toml");
        fs::write(&config_path, config).expect("write config");
        config_path
    }

    fn expected_source_path(storage_root: &Path) -> PathBuf {
        storage_root.join(SKILL_ID).join("packages").join("browser")
    }

    fn expected_target_path(target_root: &Path) -> PathBuf {
        target_root.join(SKILL_ID)
    }

    fn as_file_url(path: &Path) -> String {
        format!("file://{}", path.display())
    }

    fn toml_escape(path: &Path) -> String {
        path.display().to_string().replace('\\', "\\\\")
    }

    fn toml_escape_str(value: &str) -> String {
        value.replace('\\', "\\\\").replace('"', "\\\"")
    }

    fn resolved_symlink(path: &Path) -> PathBuf {
        let raw = fs::read_link(path).expect("read symlink");
        if raw.is_absolute() {
            raw
        } else {
            path.parent()
                .expect("parent")
                .join(raw)
                .canonicalize()
                .expect("canonicalize")
        }
    }

    fn create_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, target)
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(source, target)
        }
    }
}
