use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use eden_skills_core::adapter::create_adapter;
use eden_skills_core::config::{config_dir_from_path, Config};
use eden_skills_core::error::EdenError;
use eden_skills_core::lock::{
    compute_lock_diff, lock_path_for_config, read_lock_file, LockSkillEntry, SkillDiffStatus,
};
use eden_skills_core::paths::{known_default_agent_paths, resolve_path_string};
use eden_skills_core::plan::{build_plan, Action};
use eden_skills_core::reactor::SkillReactor;
use eden_skills_core::safety::{analyze_skills, persist_reports, SkillSafetyReport};
use eden_skills_core::source::sync_sources_async_with_reactor;
use eden_skills_core::verify::verify_config_state;

use super::common::{
    apply_plan_item, block_on_command_future, ensure_docker_available_for_targets,
    ensure_git_available, load_config_with_context, print_safety_summary,
    print_source_sync_summary, read_head_sha, remove_path, resolve_config_path,
    resolve_effective_reactor_concurrency, resolve_registry_mode_skills_for_execution,
    source_sync_failure_error, write_lock_for_config, write_lock_for_config_with_commits,
};

use super::CommandOptions;

pub fn apply(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    block_on_command_future(apply_async(config_path, options, None))
}

pub async fn apply_async(
    config_path: &str,
    options: CommandOptions,
    concurrency_override: Option<usize>,
) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, options.strict)?;
    let concurrency = resolve_effective_reactor_concurrency(
        concurrency_override,
        loaded.config.reactor.concurrency,
        "apply.concurrency",
    )?;
    let reactor = SkillReactor::new(concurrency).map_err(EdenError::from)?;
    let config_dir = config_dir_from_path(config_path);
    let execution_config =
        resolve_registry_mode_skills_for_execution(config_path, &loaded.config, &config_dir)?;
    if !execution_config.skills.is_empty() {
        ensure_git_available()?;
    }

    let lock_path = lock_path_for_config(config_path);
    let lock = read_lock_file(&lock_path)?;
    let diff = compute_lock_diff(&execution_config, &lock, &config_dir)?;
    ensure_docker_available_for_targets(diff.removed.iter().flat_map(|entry| {
        entry
            .targets
            .iter()
            .map(|target| target.environment.as_str())
    }))?;

    let sync_config = filter_config_for_sync(&execution_config, &diff);
    let sync_summary = sync_sources_async_with_reactor(&sync_config, &config_dir, reactor).await?;
    print_source_sync_summary(&sync_summary);
    let safety_reports = analyze_skills(&execution_config, &config_dir)?;
    persist_reports(&safety_reports)?;
    print_safety_summary(&safety_reports);
    if let Some(err) = source_sync_failure_error(&sync_summary) {
        return Err(err);
    }

    let removed_count =
        uninstall_orphaned_lock_entries(&diff.removed, &config_dir, &execution_config.storage_root)
            .await?;

    let no_exec_skill_ids = no_exec_skill_ids(&safety_reports);
    let plan = build_plan(&execution_config, &config_dir)?;

    let mut created = 0usize;
    let mut updated = 0usize;
    let mut noops = 0usize;
    let mut conflicts = 0usize;
    let mut skipped_no_exec = 0usize;

    for item in &plan {
        match item.action {
            Action::Create => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    skipped_no_exec += 1;
                    continue;
                }
                apply_plan_item(item)?;
                created += 1;
            }
            Action::Update => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    skipped_no_exec += 1;
                    continue;
                }
                apply_plan_item(item)?;
                updated += 1;
            }
            Action::Noop => {
                noops += 1;
            }
            Action::Conflict => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    skipped_no_exec += 1;
                    continue;
                }
                conflicts += 1;
            }
            Action::Remove => {}
        }
    }

    println!(
        "apply summary: create={created} update={updated} noop={noops} conflict={conflicts} skipped_no_exec={skipped_no_exec} removed={removed_count}"
    );

    if options.strict && conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "strict mode blocked apply: {conflicts} conflict entries"
        )));
    }

    let verify_issues = verify_config_state(&execution_config, &config_dir)?;
    if !verify_issues.is_empty() {
        return Err(EdenError::Runtime(format!(
            "post-apply verification failed with {} issue(s); first: [{}] {} {}",
            verify_issues.len(),
            verify_issues[0].check,
            verify_issues[0].skill_id,
            verify_issues[0].message
        )));
    }

    let resolved_commits = collect_resolved_commits(&execution_config, &config_dir);
    write_lock_for_config_with_commits(
        config_path,
        &execution_config,
        &config_dir,
        &resolved_commits,
    )?;

    println!("apply verification: ok");
    Ok(())
}

pub fn repair(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    block_on_command_future(repair_async(config_path, options, None))
}

pub async fn repair_async(
    config_path: &str,
    options: CommandOptions,
    concurrency_override: Option<usize>,
) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, options.strict)?;
    let concurrency = resolve_effective_reactor_concurrency(
        concurrency_override,
        loaded.config.reactor.concurrency,
        "repair.concurrency",
    )?;
    let reactor = SkillReactor::new(concurrency).map_err(EdenError::from)?;
    let config_dir = config_dir_from_path(config_path);
    let execution_config =
        resolve_registry_mode_skills_for_execution(config_path, &loaded.config, &config_dir)?;
    if !execution_config.skills.is_empty() {
        ensure_git_available()?;
    }
    let sync_summary =
        sync_sources_async_with_reactor(&execution_config, &config_dir, reactor).await?;
    print_source_sync_summary(&sync_summary);
    let safety_reports = analyze_skills(&execution_config, &config_dir)?;
    persist_reports(&safety_reports)?;
    print_safety_summary(&safety_reports);
    if let Some(err) = source_sync_failure_error(&sync_summary) {
        return Err(err);
    }

    let no_exec_skill_ids = no_exec_skill_ids(&safety_reports);
    let plan = build_plan(&execution_config, &config_dir)?;

    let mut repaired = 0usize;
    let mut skipped_conflicts = 0usize;
    let mut skipped_no_exec = 0usize;

    for item in &plan {
        match item.action {
            Action::Create | Action::Update => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    skipped_no_exec += 1;
                    continue;
                }
                apply_plan_item(item)?;
                repaired += 1;
            }
            Action::Conflict => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    skipped_no_exec += 1;
                    continue;
                }
                skipped_conflicts += 1;
            }
            Action::Noop | Action::Remove => {}
        }
    }

    println!(
        "repair summary: repaired={repaired} skipped_conflicts={skipped_conflicts} skipped_no_exec={skipped_no_exec}"
    );

    if options.strict && skipped_conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "repair skipped {skipped_conflicts} conflict entries in strict mode"
        )));
    }

    let verify_issues = verify_config_state(&execution_config, &config_dir)?;
    if !verify_issues.is_empty() {
        return Err(EdenError::Runtime(format!(
            "post-repair verification failed with {} issue(s); first: [{}] {} {}",
            verify_issues.len(),
            verify_issues[0].check,
            verify_issues[0].skill_id,
            verify_issues[0].message
        )));
    }

    write_lock_for_config(config_path, &execution_config, &config_dir)?;

    println!("repair verification: ok");
    Ok(())
}

fn no_exec_skill_ids(reports: &[SkillSafetyReport]) -> HashSet<&str> {
    reports
        .iter()
        .filter(|r| r.no_exec_metadata_only)
        .map(|r| r.skill_id.as_str())
        .collect()
}

/// Create a config subset containing only skills that need source sync
/// (Added or Changed per lock diff). Unchanged skills are skipped unless
/// their storage directory is missing (reclassified as needing sync).
fn filter_config_for_sync(
    config: &Config,
    diff: &eden_skills_core::lock::LockDiffResult,
) -> Config {
    let mut sync_skills = Vec::new();
    for skill in &config.skills {
        let status = diff.statuses.get(&skill.id);
        if !matches!(status, Some(SkillDiffStatus::Unchanged)) {
            sync_skills.push(skill.clone());
        }
    }
    Config {
        version: config.version,
        storage_root: config.storage_root.clone(),
        reactor: config.reactor,
        skills: sync_skills,
    }
}

fn collect_resolved_commits(config: &Config, config_dir: &Path) -> HashMap<String, String> {
    let storage_root = match resolve_path_string(&config.storage_root, config_dir) {
        Ok(p) => p,
        Err(_) => return HashMap::new(),
    };
    let mut commits = HashMap::new();
    for skill in &config.skills {
        let repo_dir = storage_root.join(&skill.id);
        if let Some(sha) = read_head_sha(&repo_dir) {
            commits.insert(skill.id.clone(), sha);
        }
    }
    commits
}

async fn uninstall_orphaned_lock_entries(
    removed: &[LockSkillEntry],
    config_dir: &Path,
    storage_root: &str,
) -> Result<usize, EdenError> {
    let mut count = 0;
    for entry in removed {
        for target in &entry.targets {
            let target_path = PathBuf::from(&target.path);
            let adapter = create_adapter(&target.environment).map_err(EdenError::from)?;
            adapter
                .uninstall(&target_path)
                .await
                .map_err(EdenError::from)?;
        }
        remove_from_known_local_agent_targets(&entry.id, config_dir)?;
        let resolved_storage = resolve_path_string(storage_root, config_dir)?;
        let storage_dir = resolved_storage.join(&entry.id);
        if storage_dir.exists() {
            fs::remove_dir_all(&storage_dir)?;
        }
        count += 1;
    }
    Ok(count)
}

fn remove_from_known_local_agent_targets(
    skill_id: &str,
    config_dir: &Path,
) -> Result<(), EdenError> {
    let mut scanned_roots = HashSet::new();
    for raw_root in known_default_agent_paths() {
        let resolved_root = resolve_path_string(raw_root, config_dir)?;
        if !scanned_roots.insert(resolved_root.clone()) {
            continue;
        }
        let candidate = eden_skills_core::paths::normalize_lexical(&resolved_root.join(skill_id));
        if fs::symlink_metadata(&candidate).is_ok() {
            remove_path(&candidate)?;
        }
    }
    Ok(())
}
