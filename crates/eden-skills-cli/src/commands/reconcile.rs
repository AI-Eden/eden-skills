//! State reconciliation: `apply` and `repair` command implementations.
//!
//! Both commands follow the same lifecycle: source sync → safety analysis →
//! orphan removal → plan execution → verification → lock file write.
//! `repair` additionally force-reinstalls every target regardless of drift
//! status, ensuring convergence from any starting state.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use eden_skills_core::adapter::create_adapter;
use eden_skills_core::config::{config_dir_from_path, Config};
use eden_skills_core::error::EdenError;
use eden_skills_core::lock::{
    compute_lock_diff, lock_path_for_config, read_lock_file, LockSkillEntry, SkillDiffStatus,
};
use eden_skills_core::managed::{external_install_origin, ManagedSource};
use eden_skills_core::paths::{known_default_agent_paths, resolve_path_string};
use eden_skills_core::plan::{build_plan, Action};
use eden_skills_core::reactor::SkillReactor;
use eden_skills_core::safety::{analyze_skills, persist_reports, SkillSafetyReport};
use eden_skills_core::source::{
    repo_cache_key, resolve_skill_storage_root, sync_sources_async_with_reactor,
    sync_sources_async_with_reactor_skipping_repos,
};
use eden_skills_core::verify::verify_config_state;
use owo_colors::OwoColorize;

use super::common::{
    apply_plan_item, block_on_command_future, ensure_docker_available_for_targets,
    ensure_git_available, load_config_with_context, print_safety_summary_human,
    print_source_sync_summary_human, print_warning, read_head_sha, remove_path,
    resolve_config_path, resolve_effective_reactor_concurrency,
    resolve_registry_mode_skills_for_execution, source_sync_failure_error, style_count_for_action,
    write_lock_for_config, write_lock_for_config_with_commits,
};

use super::CommandOptions;
use crate::ui::{StatusSymbol, UiContext};
use eden_skills_core::adapter::{read_managed_manifest, write_managed_manifest};

#[derive(Debug)]
struct AppliedInstallTargetLine {
    skill_id: String,
    target_path: String,
    mode: String,
}

/// Synchronous wrapper around [`apply_async`] using a single-threaded runtime.
///
/// # Errors
///
/// Returns [`EdenError`] if any phase of the apply lifecycle fails.
pub fn apply(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    block_on_command_future(apply_async(config_path, options, None, false))
}

/// Reconcile installed state with the declared configuration.
///
/// Executes the full apply lifecycle: source sync → safety analysis →
/// orphan removal → plan execution → verification → lock write.
/// Only targets that have drifted from the plan are reinstalled.
///
/// # Errors
///
/// Returns [`EdenError`] on config load failure, source sync errors,
/// adapter I/O errors, or verification failures in strict mode.
pub async fn apply_async(
    config_path: &str,
    options: CommandOptions,
    concurrency_override: Option<usize>,
    force: bool,
) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, options.strict)?;
    let ui = UiContext::from_env(options.json);
    for warning in &loaded.warnings {
        print_warning(&ui, warning);
    }
    let concurrency = resolve_effective_reactor_concurrency(
        concurrency_override,
        loaded.config.reactor.concurrency,
        "apply.concurrency",
    )?;
    let reactor = SkillReactor::new(concurrency).map_err(EdenError::from)?;
    let config_dir = config_dir_from_path(config_path);
    let execution_config =
        resolve_registry_mode_skills_for_execution(config_path, &loaded.config, &config_dir, &ui)?;
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

    let skip_repos = skip_repo_cache_keys_for_apply(&execution_config, &diff);
    let sync_summary = sync_sources_async_with_reactor_skipping_repos(
        &execution_config,
        &config_dir,
        reactor,
        &skip_repos,
    )
    .await?;
    print_source_sync_summary_human(&ui, &sync_summary);
    let safety_reports = analyze_skills(&execution_config, &config_dir)?;
    persist_reports(&safety_reports)?;
    print_safety_summary_human(&ui, &safety_reports);
    if let Some(err) = source_sync_failure_error(&sync_summary) {
        return Err(err);
    }

    let removed_skill_ids =
        uninstall_orphaned_lock_entries(&diff.removed, &config_dir, &execution_config.storage_root)
            .await?;
    print_remove_lines(&ui, &removed_skill_ids);
    let removed_count = removed_skill_ids.len();

    let no_exec_skill_ids = no_exec_skill_ids(&safety_reports);
    let ownership_blocked_skill_ids =
        collect_docker_takeover_skips(&execution_config, &config_dir, options.json, force).await?;
    let plan = build_plan(&execution_config, &config_dir)?;
    let mut applied_targets: Vec<AppliedInstallTargetLine> = Vec::new();
    let mut skipped_skill_ids: Vec<String> = Vec::new();

    let mut created = 0usize;
    let mut updated = 0usize;
    let mut noops = 0usize;
    let mut conflicts = 0usize;

    for item in &plan {
        match item.action {
            Action::Create => {
                if no_exec_skill_ids.contains(item.skill_id.as_str())
                    || ownership_blocked_skill_ids.contains(item.skill_id.as_str())
                {
                    push_skipped_skill(&mut skipped_skill_ids, &item.skill_id);
                    continue;
                }
                apply_plan_item(item)?;
                created += 1;
                applied_targets.push(AppliedInstallTargetLine {
                    skill_id: item.skill_id.clone(),
                    target_path: item.target_path.clone(),
                    mode: item.install_mode.as_str().to_string(),
                });
            }
            Action::Update => {
                if no_exec_skill_ids.contains(item.skill_id.as_str())
                    || ownership_blocked_skill_ids.contains(item.skill_id.as_str())
                {
                    push_skipped_skill(&mut skipped_skill_ids, &item.skill_id);
                    continue;
                }
                apply_plan_item(item)?;
                updated += 1;
                applied_targets.push(AppliedInstallTargetLine {
                    skill_id: item.skill_id.clone(),
                    target_path: item.target_path.clone(),
                    mode: item.install_mode.as_str().to_string(),
                });
            }
            Action::Noop => {
                noops += 1;
            }
            Action::Conflict => {
                if no_exec_skill_ids.contains(item.skill_id.as_str())
                    || ownership_blocked_skill_ids.contains(item.skill_id.as_str())
                {
                    push_skipped_skill(&mut skipped_skill_ids, &item.skill_id);
                    continue;
                }
                conflicts += 1;
            }
            Action::Remove => {}
        }
    }
    if force {
        reclaim_docker_ownership(&execution_config, &config_dir).await?;
    }
    print_install_result_lines(&ui, &applied_targets, &skipped_skill_ids);

    println!(
        "{}  {} {} created, {} updated, {} noop, {} conflicts, {} removed",
        ui.action_prefix("Summary"),
        ui.status_symbol(StatusSymbol::Success),
        style_count_for_action(&ui, "create", created),
        style_count_for_action(&ui, "update", updated),
        style_count_for_action(&ui, "noop", noops),
        style_count_for_action(&ui, "conflict", conflicts),
        style_count_for_action(&ui, "remove", removed_count),
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

    println!(
        "  {} Verification passed",
        ui.status_symbol(StatusSymbol::Success)
    );
    Ok(())
}

/// Synchronous wrapper around [`repair_async`] using a single-threaded runtime.
///
/// # Errors
///
/// Returns [`EdenError`] if any phase of the repair lifecycle fails.
pub fn repair(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    block_on_command_future(repair_async(config_path, options, None, false))
}

/// Force-reinstall every target to converge from any starting state.
///
/// Follows the same lifecycle as [`apply_async`] but reinstalls all
/// targets regardless of drift status, ensuring a clean slate.
///
/// # Errors
///
/// Returns [`EdenError`] on config load failure, source sync errors,
/// adapter I/O errors, or verification failures in strict mode.
pub async fn repair_async(
    config_path: &str,
    options: CommandOptions,
    concurrency_override: Option<usize>,
    force: bool,
) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, options.strict)?;
    let ui = UiContext::from_env(options.json);
    for warning in &loaded.warnings {
        print_warning(&ui, warning);
    }
    let concurrency = resolve_effective_reactor_concurrency(
        concurrency_override,
        loaded.config.reactor.concurrency,
        "repair.concurrency",
    )?;
    let reactor = SkillReactor::new(concurrency).map_err(EdenError::from)?;
    let config_dir = config_dir_from_path(config_path);
    let execution_config =
        resolve_registry_mode_skills_for_execution(config_path, &loaded.config, &config_dir, &ui)?;
    if !execution_config.skills.is_empty() {
        ensure_git_available()?;
    }
    let sync_summary =
        sync_sources_async_with_reactor(&execution_config, &config_dir, reactor).await?;
    print_source_sync_summary_human(&ui, &sync_summary);
    let safety_reports = analyze_skills(&execution_config, &config_dir)?;
    persist_reports(&safety_reports)?;
    print_safety_summary_human(&ui, &safety_reports);
    if let Some(err) = source_sync_failure_error(&sync_summary) {
        return Err(err);
    }

    let no_exec_skill_ids = no_exec_skill_ids(&safety_reports);
    let ownership_blocked_skill_ids =
        collect_docker_takeover_skips(&execution_config, &config_dir, options.json, force).await?;
    let plan = build_plan(&execution_config, &config_dir)?;
    let mut applied_targets: Vec<AppliedInstallTargetLine> = Vec::new();
    let mut skipped_skill_ids: Vec<String> = Vec::new();

    let mut created = 0usize;
    let mut updated = 0usize;
    let mut noops = 0usize;
    let mut conflicts = 0usize;

    for item in &plan {
        match item.action {
            Action::Create => {
                if no_exec_skill_ids.contains(item.skill_id.as_str())
                    || ownership_blocked_skill_ids.contains(item.skill_id.as_str())
                {
                    push_skipped_skill(&mut skipped_skill_ids, &item.skill_id);
                    continue;
                }
                apply_plan_item(item)?;
                created += 1;
                applied_targets.push(AppliedInstallTargetLine {
                    skill_id: item.skill_id.clone(),
                    target_path: item.target_path.clone(),
                    mode: item.install_mode.as_str().to_string(),
                });
            }
            Action::Update => {
                if no_exec_skill_ids.contains(item.skill_id.as_str())
                    || ownership_blocked_skill_ids.contains(item.skill_id.as_str())
                {
                    push_skipped_skill(&mut skipped_skill_ids, &item.skill_id);
                    continue;
                }
                apply_plan_item(item)?;
                updated += 1;
                applied_targets.push(AppliedInstallTargetLine {
                    skill_id: item.skill_id.clone(),
                    target_path: item.target_path.clone(),
                    mode: item.install_mode.as_str().to_string(),
                });
            }
            Action::Conflict => {
                if no_exec_skill_ids.contains(item.skill_id.as_str())
                    || ownership_blocked_skill_ids.contains(item.skill_id.as_str())
                {
                    push_skipped_skill(&mut skipped_skill_ids, &item.skill_id);
                    continue;
                }
                conflicts += 1;
            }
            Action::Noop => {
                noops += 1;
            }
            Action::Remove => {}
        }
    }
    if force {
        reclaim_docker_ownership(&execution_config, &config_dir).await?;
    }
    print_install_result_lines(&ui, &applied_targets, &skipped_skill_ids);

    println!(
        "{}  {} {} created, {} updated, {} noop, {} conflicts, {} removed",
        ui.action_prefix("Summary"),
        ui.status_symbol(StatusSymbol::Success),
        style_count_for_action(&ui, "create", created),
        style_count_for_action(&ui, "update", updated),
        style_count_for_action(&ui, "noop", noops),
        style_count_for_action(&ui, "conflict", conflicts),
        style_count_for_action(&ui, "remove", 0),
    );

    if options.strict && conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "repair skipped {conflicts} conflict entries in strict mode"
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

    println!(
        "  {} Verification passed",
        ui.status_symbol(StatusSymbol::Success)
    );
    Ok(())
}

fn no_exec_skill_ids(reports: &[SkillSafetyReport]) -> HashSet<&str> {
    reports
        .iter()
        .filter(|r| r.no_exec_metadata_only)
        .map(|r| r.skill_id.as_str())
        .collect()
}

/// Compute repo-cache keys that `apply` may skip during source sync.
///
/// Skip is decided at the repo cache level, not per skill: if any skill
/// sharing the same `(repo_url, ref)` is Added/Changed, that repo must
/// still sync. Only cache keys whose participating skills are all
/// `Unchanged` are eligible for skip.
fn skip_repo_cache_keys_for_apply(
    config: &Config,
    diff: &eden_skills_core::lock::LockDiffResult,
) -> HashSet<String> {
    let mut unchanged = HashSet::new();
    let mut must_sync = HashSet::new();
    for skill in &config.skills {
        let cache_key = repo_cache_key(&skill.source.repo, &skill.source.r#ref);
        if matches!(
            diff.statuses.get(&skill.id),
            Some(SkillDiffStatus::Unchanged)
        ) {
            unchanged.insert(cache_key);
        } else {
            must_sync.insert(cache_key);
        }
    }
    unchanged.retain(|cache_key| !must_sync.contains(cache_key));
    unchanged
}

fn collect_resolved_commits(config: &Config, config_dir: &Path) -> HashMap<String, String> {
    let storage_root = match resolve_path_string(&config.storage_root, config_dir) {
        Ok(p) => p,
        Err(_) => return HashMap::new(),
    };
    let mut commits = HashMap::new();
    for skill in &config.skills {
        let repo_dir = resolve_skill_storage_root(&storage_root, skill);
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
) -> Result<Vec<String>, EdenError> {
    let mut removed_ids = Vec::with_capacity(removed.len());
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
        removed_ids.push(entry.id.clone());
    }
    Ok(removed_ids)
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

fn push_skipped_skill(skipped_skill_ids: &mut Vec<String>, skill_id: &str) {
    if !skipped_skill_ids
        .iter()
        .any(|existing| existing == skill_id)
    {
        skipped_skill_ids.push(skill_id.to_string());
    }
}

async fn collect_docker_takeover_skips(
    config: &Config,
    config_dir: &Path,
    json_mode: bool,
    force: bool,
) -> Result<HashSet<String>, EdenError> {
    let ui = UiContext::from_env(json_mode);
    let mut skipped = HashSet::new();
    if force {
        return Ok(skipped);
    }

    for skill in &config.skills {
        for target in &skill.targets {
            if !target.environment.starts_with("docker:") {
                continue;
            }
            let target_root = resolve_path_string(
                target
                    .path
                    .as_deref()
                    .or(target.expected_path.as_deref())
                    .unwrap_or(""),
                config_dir,
            )
            .or_else(|_| resolve_path_string("~/.claude/skills", config_dir))?;
            let read_result = read_managed_manifest(&target.environment, &target_root)
                .await
                .map_err(EdenError::from)?;
            if let Some(warning) = read_result.warning {
                print_warning(&ui, &warning);
            }
            if read_result
                .manifest
                .skill(&skill.id)
                .is_some_and(|record| record.source == ManagedSource::Local)
            {
                print_warning(
                    &ui,
                    &format!(
                        "Skill '{}' was taken over by local management in container. ~> Run 'eden-skills apply --force' to reclaim, or 'eden-skills remove {}' to accept.",
                        skill.id, skill.id
                    ),
                );
                skipped.insert(skill.id.clone());
            }
        }
    }
    Ok(skipped)
}

async fn reclaim_docker_ownership(config: &Config, config_dir: &Path) -> Result<(), EdenError> {
    for skill in &config.skills {
        for target in &skill.targets {
            if !target.environment.starts_with("docker:") {
                continue;
            }
            let Ok(target_root) = resolve_path_string(
                target
                    .path
                    .as_deref()
                    .or(target.expected_path.as_deref())
                    .unwrap_or(""),
                config_dir,
            ) else {
                continue;
            };
            let read_result = read_managed_manifest(&target.environment, &target_root)
                .await
                .map_err(EdenError::from)?;
            let mut manifest = read_result.manifest;
            if manifest
                .skill(&skill.id)
                .is_some_and(|record| record.source == ManagedSource::Local)
            {
                manifest.record_install(
                    &skill.id,
                    ManagedSource::External,
                    external_install_origin(),
                );
                write_managed_manifest(&target.environment, &target_root, &manifest)
                    .await
                    .map_err(EdenError::from)?;
            }
        }
    }
    Ok(())
}

fn print_install_result_lines(
    ui: &UiContext,
    applied_targets: &[AppliedInstallTargetLine],
    skipped_skill_ids: &[String],
) {
    if applied_targets.is_empty() && skipped_skill_ids.is_empty() {
        return;
    }

    let mut install_prefix_emitted = false;
    for (skill_id, targets) in group_install_targets(applied_targets) {
        let prefix = if install_prefix_emitted {
            "          ".to_string()
        } else {
            install_prefix_emitted = true;
            format!("{}  ", ui.action_prefix("Install"))
        };
        println!(
            "{prefix}{} {}",
            ui.status_symbol(StatusSymbol::Success),
            style_skill_id(ui, &skill_id),
        );
        for (index, target) in targets.iter().enumerate() {
            let connector = if index + 1 == targets.len() {
                "└─"
            } else {
                "├─"
            };
            println!(
                "             {} {} {}",
                style_tree_connector(ui, connector),
                ui.styled_path(&target.target_path),
                style_mode_label(ui, &target.mode)
            );
        }
    }

    for skill_id in skipped_skill_ids {
        let prefix = if install_prefix_emitted {
            "          ".to_string()
        } else {
            install_prefix_emitted = true;
            format!("{}  ", ui.action_prefix("Install"))
        };
        println!(
            "{prefix}{} {} (skipped: metadata-only)",
            ui.status_symbol(StatusSymbol::Skipped),
            style_skill_id(ui, skill_id),
        );
    }
}

fn group_install_targets(
    targets: &[AppliedInstallTargetLine],
) -> Vec<(String, Vec<&AppliedInstallTargetLine>)> {
    let mut groups: Vec<(String, Vec<&AppliedInstallTargetLine>)> = Vec::new();
    for target in targets {
        if let Some(group) = groups.last_mut().filter(|(id, _)| id == &target.skill_id) {
            group.1.push(target);
        } else {
            groups.push((target.skill_id.clone(), vec![target]));
        }
    }
    groups
}

fn print_remove_lines(ui: &UiContext, removed_skill_ids: &[String]) {
    if removed_skill_ids.is_empty() {
        return;
    }

    let mut remove_prefix_emitted = false;
    for skill_id in removed_skill_ids {
        if remove_prefix_emitted {
            println!(
                "          {} {}",
                ui.status_symbol(StatusSymbol::Success),
                style_skill_id(ui, skill_id)
            );
        } else {
            remove_prefix_emitted = true;
            println!(
                "{}  {} {}",
                ui.action_prefix("Remove"),
                ui.status_symbol(StatusSymbol::Success),
                style_skill_id(ui, skill_id)
            );
        }
    }
}

fn style_skill_id(ui: &UiContext, skill_id: &str) -> String {
    if ui.colors_enabled() {
        skill_id.bold().to_string()
    } else {
        skill_id.to_string()
    }
}

fn style_mode_label(ui: &UiContext, mode: &str) -> String {
    let raw = format!("({mode})");
    if ui.colors_enabled() {
        raw.dimmed().to_string()
    } else {
        raw
    }
}

fn style_tree_connector(ui: &UiContext, connector: &str) -> String {
    if ui.colors_enabled() {
        connector.dimmed().to_string()
    } else {
        connector.to_string()
    }
}
