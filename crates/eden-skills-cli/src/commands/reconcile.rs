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
use eden_skills_core::paths::{known_default_agent_paths, resolve_path_string};
use eden_skills_core::plan::{build_plan, Action};
use eden_skills_core::reactor::SkillReactor;
use eden_skills_core::safety::{analyze_skills, persist_reports, SkillSafetyReport};
use eden_skills_core::source::sync_sources_async_with_reactor;
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

/// Synchronous wrapper around [`apply_async`] using a single-threaded runtime.
///
/// # Errors
///
/// Returns [`EdenError`] if any phase of the apply lifecycle fails.
pub fn apply(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    block_on_command_future(apply_async(config_path, options, None))
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

    let sync_config = filter_config_for_sync(&execution_config, &diff);
    let sync_summary = sync_sources_async_with_reactor(&sync_config, &config_dir, reactor).await?;
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
    let plan = build_plan(&execution_config, &config_dir)?;
    let mut install_prefix_emitted = false;

    let mut created = 0usize;
    let mut updated = 0usize;
    let mut noops = 0usize;
    let mut conflicts = 0usize;

    for item in &plan {
        match item.action {
            Action::Create => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    print_install_skipped_line(&ui, &mut install_prefix_emitted, &item.skill_id);
                    continue;
                }
                apply_plan_item(item)?;
                created += 1;
                print_install_applied_line(
                    &ui,
                    &mut install_prefix_emitted,
                    &item.skill_id,
                    &item.target_path,
                    item.install_mode.as_str(),
                );
            }
            Action::Update => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    print_install_skipped_line(&ui, &mut install_prefix_emitted, &item.skill_id);
                    continue;
                }
                apply_plan_item(item)?;
                updated += 1;
                print_install_applied_line(
                    &ui,
                    &mut install_prefix_emitted,
                    &item.skill_id,
                    &item.target_path,
                    item.install_mode.as_str(),
                );
            }
            Action::Noop => {
                noops += 1;
            }
            Action::Conflict => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    print_install_skipped_line(&ui, &mut install_prefix_emitted, &item.skill_id);
                    continue;
                }
                conflicts += 1;
            }
            Action::Remove => {}
        }
    }

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
    block_on_command_future(repair_async(config_path, options, None))
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
    let plan = build_plan(&execution_config, &config_dir)?;
    let mut install_prefix_emitted = false;

    let mut created = 0usize;
    let mut updated = 0usize;
    let mut noops = 0usize;
    let mut conflicts = 0usize;

    for item in &plan {
        match item.action {
            Action::Create => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    print_install_skipped_line(&ui, &mut install_prefix_emitted, &item.skill_id);
                    continue;
                }
                apply_plan_item(item)?;
                created += 1;
                print_install_applied_line(
                    &ui,
                    &mut install_prefix_emitted,
                    &item.skill_id,
                    &item.target_path,
                    item.install_mode.as_str(),
                );
            }
            Action::Update => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    print_install_skipped_line(&ui, &mut install_prefix_emitted, &item.skill_id);
                    continue;
                }
                apply_plan_item(item)?;
                updated += 1;
                print_install_applied_line(
                    &ui,
                    &mut install_prefix_emitted,
                    &item.skill_id,
                    &item.target_path,
                    item.install_mode.as_str(),
                );
            }
            Action::Conflict => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    print_install_skipped_line(&ui, &mut install_prefix_emitted, &item.skill_id);
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

fn print_install_applied_line(
    ui: &UiContext,
    install_prefix_emitted: &mut bool,
    skill_id: &str,
    target_path: &str,
    mode: &str,
) {
    let prefix = if *install_prefix_emitted {
        "          ".to_string()
    } else {
        *install_prefix_emitted = true;
        format!("{}  ", ui.action_prefix("Install"))
    };
    println!(
        "{prefix}{} {} {} {} {}",
        ui.status_symbol(StatusSymbol::Success),
        style_skill_id(ui, skill_id),
        style_arrow(ui),
        ui.styled_path(target_path),
        style_mode_label(ui, mode),
    );
}

fn print_install_skipped_line(ui: &UiContext, install_prefix_emitted: &mut bool, skill_id: &str) {
    let prefix = if *install_prefix_emitted {
        "          ".to_string()
    } else {
        *install_prefix_emitted = true;
        format!("{}  ", ui.action_prefix("Install"))
    };
    println!(
        "{prefix}{} {} (skipped: metadata-only)",
        ui.status_symbol(StatusSymbol::Skipped),
        style_skill_id(ui, skill_id),
    );
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

fn style_arrow(ui: &UiContext) -> String {
    let arrow = "→";
    if ui.colors_enabled() {
        arrow.dimmed().to_string()
    } else {
        arrow.to_string()
    }
}
