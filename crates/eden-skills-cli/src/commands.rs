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
