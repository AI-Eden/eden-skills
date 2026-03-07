//! Registry refresh via the `update` command.
//!
//! Resolves configured registries, clones or fetches each in parallel
//! using the reactor, and records sync markers. Results are rendered as
//! a table with per-registry status and a timing footer.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use comfy_table::{ColumnConstraint, Width};
use eden_skills_core::config::{config_dir_from_path, is_registry_mode_repo, Config};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::resolve_path_string;
use eden_skills_core::plan::{build_plan, Action};
use eden_skills_core::reactor::SkillReactor;
use eden_skills_core::registry::{parse_registry_specs_from_toml, sort_registry_specs_by_priority};
use eden_skills_core::safety::{analyze_skills, persist_reports, SkillSafetyReport};
use eden_skills_core::source::{
    repo_cache_key, resolve_skill_storage_root, sync_sources_async_with_reactor,
};
use eden_skills_core::verify::verify_config_state;
use owo_colors::OwoColorize;

use super::common::{
    apply_plan_item, ensure_git_available, load_config_with_context, print_safety_summary_human,
    print_source_sync_summary_human, print_warning, read_head_sha, resolve_config_path,
    resolve_effective_reactor_concurrency, run_git_command, source_sync_failure_error,
    write_lock_for_config_with_commits, REGISTRY_SYNC_MARKER_FILE,
};
use super::UpdateRequest;
use crate::ui::{StatusSymbol, UiContext};

#[derive(Debug, Clone, PartialEq, Eq)]
enum RegistrySyncStatus {
    Cloned,
    Updated,
    Skipped,
    Failed,
}

impl RegistrySyncStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Cloned => "cloned",
            Self::Updated => "updated",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SkillRefreshStatus {
    NewCommit,
    UpToDate,
    Missing,
    Failed,
}

impl SkillRefreshStatus {
    fn table_label(self) -> &'static str {
        match self {
            Self::NewCommit => "new commit",
            Self::UpToDate => "up-to-date",
            Self::Missing => "missing",
            Self::Failed => "failed",
        }
    }

    fn json_label(self) -> &'static str {
        match self {
            Self::NewCommit => "new-commit",
            Self::UpToDate => "up-to-date",
            Self::Missing => "missing",
            Self::Failed => "failed",
        }
    }

    fn requires_apply(self) -> bool {
        matches!(self, Self::NewCommit | Self::Missing)
    }
}

#[derive(Debug, Clone)]
struct RegistrySyncTask {
    name: String,
    url: String,
    local_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct RegistrySyncResult {
    name: String,
    status: RegistrySyncStatus,
    url: String,
    detail: Option<String>,
}

#[derive(Debug, Clone)]
struct SkillRefreshTask {
    skill_ids: Vec<String>,
    reference: String,
    local_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct SkillRefreshResult {
    id: String,
    status: SkillRefreshStatus,
    local_sha: Option<String>,
    remote_sha: Option<String>,
    detail: Option<String>,
    applied: bool,
}

#[derive(Debug, Clone)]
struct AppliedInstallTargetLine {
    skill_id: String,
    target_path: String,
    mode: String,
}

#[derive(Debug, Default)]
struct ApplyOutcome {
    applied_skill_ids: HashSet<String>,
}

/// Refresh all configured registries to their latest versions.
///
/// Resolves registry specs from the config, clones new registries or
/// fetches updates for existing ones using reactor-based concurrency,
/// and writes sync marker timestamps. Results are displayed as a table.
///
/// # Errors
///
/// Returns [`EdenError`] on config load failure, git unavailability,
/// or reactor initialization errors.
pub async fn update_async(req: UpdateRequest) -> Result<(), EdenError> {
    let ui = UiContext::from_env(req.options.json);
    let config_path_buf = resolve_config_path(&req.config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, req.options.strict)?;
    for warning in loaded.warnings {
        super::common::print_warning(&ui, &warning);
    }

    let raw_toml = fs::read_to_string(config_path)?;
    let registry_specs = sort_registry_specs_by_priority(
        &parse_registry_specs_from_toml(&raw_toml).map_err(EdenError::from)?,
    );
    let config_dir = config_dir_from_path(config_path);
    let storage_root = resolve_path_string(&loaded.config.storage_root, &config_dir)?;
    let mode_a_tasks = build_mode_a_refresh_tasks(&loaded.config, &storage_root);
    let has_registries = !registry_specs.is_empty();
    let has_mode_a_skills = !mode_a_tasks.is_empty();

    if !has_registries && !has_mode_a_skills {
        if req.options.json {
            let payload = serde_json::json!({
                "registries": Vec::<serde_json::Value>::new(),
                "skills": Vec::<serde_json::Value>::new(),
                "failed": 0,
                "elapsed_ms": 0,
            });
            let encoded = serde_json::to_string_pretty(&payload).map_err(|err| {
                EdenError::Runtime(format!("failed to encode update json: {err}"))
            })?;
            println!("{encoded}");
        } else {
            print_empty_update_guidance(&ui);
        }
        return Ok(());
    }

    let concurrency = resolve_effective_reactor_concurrency(
        req.concurrency,
        loaded.config.reactor.concurrency,
        "update.concurrency",
    )?;
    ensure_git_available()?;

    let registries_root = storage_root.join("registries");
    let started = Instant::now();
    let mut registry_results = Vec::new();
    if has_registries {
        tokio::fs::create_dir_all(&registries_root).await?;
        let tasks = registry_specs
            .into_iter()
            .map(|spec| {
                let name = spec.name;
                RegistrySyncTask {
                    name: name.clone(),
                    url: spec.url,
                    local_dir: registries_root.join(name),
                }
            })
            .collect::<Vec<_>>();

        let reactor = SkillReactor::new(concurrency).map_err(EdenError::from)?;
        let outcomes = reactor
            .run_phase_a(tasks, move |task| {
                let reactor = reactor;
                async move { sync_registry_task(task, reactor).await }
            })
            .await
            .map_err(EdenError::from)?;
        for outcome in outcomes {
            match outcome.result {
                Ok(result) | Err(result) => registry_results.push(result),
            }
        }
        registry_results.sort_by(|left, right| left.name.cmp(&right.name));
    }

    let mut skill_results = if has_mode_a_skills {
        refresh_mode_a_skills(mode_a_tasks, concurrency).await?
    } else {
        Vec::new()
    };
    let mut pending_skill_ids = skill_results
        .iter()
        .filter(|result| result.status.requires_apply())
        .map(|result| result.id.clone())
        .collect::<Vec<_>>();
    pending_skill_ids.sort();
    pending_skill_ids.dedup();

    if !req.options.json {
        let printed_sections =
            print_update_refresh_sections(&ui, &registry_results, &skill_results);
        if printed_sections && req.apply && !pending_skill_ids.is_empty() {
            println!();
        }
    }

    let mut apply_outcome = ApplyOutcome::default();
    if req.apply && !pending_skill_ids.is_empty() {
        apply_outcome = apply_refreshed_skills(
            config_path,
            &loaded.config,
            &config_dir,
            &pending_skill_ids,
            concurrency,
            &ui,
            !req.options.json,
        )
        .await?;
    }
    for result in &mut skill_results {
        if apply_outcome.applied_skill_ids.contains(&result.id) {
            result.applied = true;
        }
    }

    let elapsed_ms = started.elapsed().as_millis() as u64;
    if req.options.json {
        print_update_json(&registry_results, &skill_results, elapsed_ms, req.apply)?;
    } else {
        println!();
        print_update_summary(
            &ui,
            &registry_results,
            &skill_results,
            req.apply,
            elapsed_ms,
        );
    }

    Ok(())
}

fn print_empty_update_guidance(ui: &UiContext) {
    println!(
        "{}  no skills or registries configured",
        ui.action_prefix("Update")
    );
    println!();
    println!(
        "  {} Run 'eden-skills install <owner/repo>' to get started.",
        ui.hint_prefix()
    );
}

fn print_update_refresh_sections(
    ui: &UiContext,
    registry_results: &[RegistrySyncResult],
    skill_results: &[SkillRefreshResult],
) -> bool {
    let mut printed = false;
    if !registry_results.is_empty() {
        println!(
            "{}  {} registries synced",
            ui.action_prefix("Update"),
            registry_results.len()
        );
        println!();

        let mut table = ui.table(&["Registry", "Status", "Detail"]);
        if let Some(column) = table.column_mut(1) {
            column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(10)));
        }
        for result in registry_results {
            table.add_row(vec![
                result.name.clone(),
                registry_status_cell(ui, &result.status),
                style_detail_cell(ui, result.detail.as_deref()),
            ]);
        }
        println!("{table}");
        printed = true;
    }

    if !skill_results.is_empty() {
        if printed {
            println!();
        }
        println!(
            "{}  {} skills checked",
            ui.action_prefix("Refresh"),
            skill_results.len()
        );
        println!();
        let mut table = ui.table(&["Skill", "Status"]);
        if let Some(column) = table.column_mut(1) {
            column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(12)));
        }
        for result in skill_results {
            table.add_row(vec![
                ui.styled_skill_id(&result.id),
                skill_refresh_status_cell(ui, result.status),
            ]);
        }
        println!("{table}");
        printed = true;
    }

    printed
}

fn print_update_summary(
    ui: &UiContext,
    registry_results: &[RegistrySyncResult],
    skill_results: &[SkillRefreshResult],
    apply_requested: bool,
    elapsed_ms: u64,
) {
    let registry_failed = registry_results
        .iter()
        .filter(|result| matches!(result.status, RegistrySyncStatus::Failed))
        .count();
    let skill_failed = skill_results
        .iter()
        .filter(|result| matches!(result.status, SkillRefreshStatus::Failed))
        .count();
    let updates_available = skill_results
        .iter()
        .filter(|result| result.status.requires_apply())
        .count();
    let applied_count = skill_results.iter().filter(|result| result.applied).count();
    let elapsed_seconds = elapsed_ms as f64 / 1000.0;
    let summary_symbol = if registry_failed == 0 && skill_failed == 0 {
        StatusSymbol::Success
    } else {
        StatusSymbol::Failure
    };

    if apply_requested {
        println!(
            "  {} {} skills updated [{elapsed_seconds:.1}s]",
            ui.status_symbol(summary_symbol),
            applied_count
        );
    } else if !skill_results.is_empty()
        && registry_results.is_empty()
        && updates_available == 0
        && skill_failed == 0
    {
        println!(
            "  {} All skills up to date [{elapsed_seconds:.1}s]",
            ui.status_symbol(StatusSymbol::Success)
        );
    } else {
        println!(
            "  {} {} registry failures, {} skills have updates [{elapsed_seconds:.1}s]",
            ui.status_symbol(summary_symbol),
            registry_failed,
            updates_available
        );
    }

    for result in registry_results
        .iter()
        .filter(|result| matches!(result.status, RegistrySyncStatus::Failed))
    {
        if let Some(detail) = &result.detail {
            print_warning(ui, &format!("registry `{}` failed: {detail}", result.name));
        }
    }
    for result in skill_results
        .iter()
        .filter(|result| matches!(result.status, SkillRefreshStatus::Failed))
    {
        if let Some(detail) = &result.detail {
            print_warning(
                ui,
                &format!("skill `{}` refresh failed: {detail}", result.id),
            );
        }
    }
    if !apply_requested && updates_available > 0 {
        println!(
            "  {} Run 'eden-skills update --apply' or 'eden-skills apply' to install.",
            ui.hint_prefix()
        );
    }
}

fn print_update_json(
    registry_results: &[RegistrySyncResult],
    skill_results: &[SkillRefreshResult],
    elapsed_ms: u64,
    include_applied: bool,
) -> Result<(), EdenError> {
    let registry_failed = registry_results
        .iter()
        .filter(|result| matches!(result.status, RegistrySyncStatus::Failed))
        .count();
    let payload = serde_json::json!({
        "registries": registry_results.iter().map(|result| {
            serde_json::json!({
                "name": result.name,
                "status": result.status.as_str(),
                "url": result.url,
                "detail": result.detail,
            })
        }).collect::<Vec<_>>(),
        "skills": skill_results.iter().map(|result| {
            if include_applied {
                serde_json::json!({
                    "id": result.id,
                    "status": result.status.json_label(),
                    "local_sha": result.local_sha,
                    "remote_sha": result.remote_sha,
                    "applied": result.applied,
                })
            } else {
                serde_json::json!({
                    "id": result.id,
                    "status": result.status.json_label(),
                    "local_sha": result.local_sha,
                    "remote_sha": result.remote_sha,
                })
            }
        }).collect::<Vec<_>>(),
        "failed": registry_failed,
        "elapsed_ms": elapsed_ms,
    });
    let encoded = serde_json::to_string_pretty(&payload)
        .map_err(|err| EdenError::Runtime(format!("failed to encode update json: {err}")))?;
    println!("{encoded}");
    Ok(())
}

fn build_mode_a_refresh_tasks(config: &Config, storage_root: &Path) -> Vec<SkillRefreshTask> {
    let mut remote_tasks = BTreeMap::new();
    let mut local_tasks = Vec::new();

    for skill in config
        .skills
        .iter()
        .filter(|skill| !is_registry_mode_repo(&skill.source.repo))
    {
        if Path::new(&skill.source.repo).is_absolute() {
            local_tasks.push(SkillRefreshTask {
                skill_ids: vec![skill.id.clone()],
                reference: skill.source.r#ref.clone(),
                local_dir: resolve_skill_storage_root(storage_root, skill),
            });
            continue;
        }

        let cache_key = repo_cache_key(&skill.source.repo, &skill.source.r#ref);
        remote_tasks
            .entry(cache_key)
            .and_modify(|task: &mut SkillRefreshTask| task.skill_ids.push(skill.id.clone()))
            .or_insert_with(|| SkillRefreshTask {
                skill_ids: vec![skill.id.clone()],
                reference: skill.source.r#ref.clone(),
                local_dir: resolve_skill_storage_root(storage_root, skill),
            });
    }

    let mut tasks = remote_tasks
        .into_values()
        .chain(local_tasks)
        .map(|mut task| {
            task.skill_ids.sort();
            task
        })
        .collect::<Vec<_>>();
    tasks.sort_by(|left, right| left.skill_ids.cmp(&right.skill_ids));
    tasks
}

async fn refresh_mode_a_skills(
    tasks: Vec<SkillRefreshTask>,
    concurrency: usize,
) -> Result<Vec<SkillRefreshResult>, EdenError> {
    if tasks.is_empty() {
        return Ok(Vec::new());
    }
    let reactor = SkillReactor::new(concurrency).map_err(EdenError::from)?;
    let outcomes = reactor
        .run_phase_a(tasks, move |task| {
            let reactor = reactor;
            async move { refresh_mode_a_task(task, reactor).await }
        })
        .await
        .map_err(EdenError::from)?;
    let mut results = Vec::new();
    for outcome in outcomes {
        match outcome.result {
            Ok(grouped_results) | Err(grouped_results) => results.extend(grouped_results),
        }
    }
    results.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(results)
}

async fn refresh_mode_a_task(
    task: SkillRefreshTask,
    reactor: SkillReactor,
) -> Result<Vec<SkillRefreshResult>, Vec<SkillRefreshResult>> {
    let failed_skill_ids = task.skill_ids.clone();
    let task_label = format!(
        "refresh source `{}`",
        describe_refresh_task(&task.skill_ids)
    );
    match reactor
        .run_blocking(&task_label, move || refresh_mode_a_task_blocking(task))
        .await
    {
        Ok(Ok(results)) => Ok(results),
        Ok(Err(results)) => Err(results),
        Err(err) => Err(build_skill_refresh_results(
            failed_skill_ids,
            SkillRefreshStatus::Failed,
            None,
            None,
            Some(err.to_string()),
        )),
    }
}

fn refresh_mode_a_task_blocking(
    task: SkillRefreshTask,
) -> Result<Result<Vec<SkillRefreshResult>, Vec<SkillRefreshResult>>, EdenError> {
    let git_dir = task.local_dir.join(".git");
    if !git_dir.exists() {
        return Ok(Ok(build_skill_refresh_results(
            task.skill_ids,
            SkillRefreshStatus::Missing,
            None,
            None,
            None,
        )));
    }

    cleanup_stale_git_locks(&task.local_dir);
    let local_sha = read_head_sha(&task.local_dir);
    record_test_git_fetch_if_configured();
    let fetch_result = run_git_command(
        Command::new("git")
            .arg("-C")
            .arg(&task.local_dir)
            .arg("fetch")
            .arg("--depth")
            .arg("1")
            .arg("origin")
            .arg(&task.reference),
        &format!("fetch source `{}`", describe_refresh_task(&task.skill_ids)),
    );
    if let Err(detail) = fetch_result {
        return Ok(Err(build_skill_refresh_results(
            task.skill_ids,
            SkillRefreshStatus::Failed,
            local_sha,
            None,
            Some(detail),
        )));
    }

    let Some(remote_sha) = read_fetch_head_sha(&task.local_dir) else {
        return Ok(Err(build_skill_refresh_results(
            task.skill_ids,
            SkillRefreshStatus::Failed,
            local_sha,
            None,
            Some("failed to read FETCH_HEAD after fetch".to_string()),
        )));
    };
    let status = if local_sha.as_deref() == Some(remote_sha.as_str()) {
        SkillRefreshStatus::UpToDate
    } else {
        SkillRefreshStatus::NewCommit
    };

    Ok(Ok(build_skill_refresh_results(
        task.skill_ids,
        status,
        local_sha,
        Some(remote_sha),
        None,
    )))
}

fn read_fetch_head_sha(repo_dir: &Path) -> Option<String> {
    let stdout = run_git_command(
        Command::new("git")
            .arg("-C")
            .arg(repo_dir)
            .arg("rev-parse")
            .arg("FETCH_HEAD"),
        &format!("read FETCH_HEAD for `{}`", repo_dir.display()),
    )
    .ok()?;
    let sha = stdout.lines().next()?.trim();
    if sha.is_empty() {
        None
    } else {
        Some(sha.to_string())
    }
}

fn describe_refresh_task(skill_ids: &[String]) -> String {
    match skill_ids {
        [] => "unknown".to_string(),
        [skill_id] => skill_id.clone(),
        [first, rest @ ..] => format!("{first} (+{} more)", rest.len()),
    }
}

fn build_skill_refresh_results(
    skill_ids: Vec<String>,
    status: SkillRefreshStatus,
    local_sha: Option<String>,
    remote_sha: Option<String>,
    detail: Option<String>,
) -> Vec<SkillRefreshResult> {
    skill_ids
        .into_iter()
        .map(|id| SkillRefreshResult {
            id,
            status,
            local_sha: local_sha.clone(),
            remote_sha: remote_sha.clone(),
            detail: detail.clone(),
            applied: false,
        })
        .collect()
}

fn cleanup_stale_git_locks(repo_dir: &Path) {
    let git_dir = repo_dir.join(".git");
    let ui = UiContext::from_env(false);
    for file_name in ["shallow.lock", "index.lock"] {
        let lock_path = git_dir.join(file_name);
        let Ok(metadata) = fs::metadata(&lock_path) else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        let Ok(age) = SystemTime::now().duration_since(modified) else {
            continue;
        };
        if age.as_secs() <= 60 {
            continue;
        }
        match fs::remove_file(&lock_path) {
            Ok(()) => print_warning(
                &ui,
                &format!(
                    "removed stale git lock `{}` before refresh",
                    lock_path.display()
                ),
            ),
            Err(err) => print_warning(
                &ui,
                &format!(
                    "failed to remove stale git lock `{}` before refresh: {err}",
                    lock_path.display()
                ),
            ),
        }
    }
}

fn record_test_git_fetch_if_configured() {
    record_test_git_event_if_configured("EDEN_SKILLS_TEST_GIT_FETCH_LOG", b"fetch\n");
}

fn record_test_git_event_if_configured(env_var: &str, line: &[u8]) {
    let Some(log_path) = std::env::var_os(env_var) else {
        return;
    };
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    else {
        return;
    };
    let _ = std::io::Write::write_all(&mut file, line);
}

async fn apply_refreshed_skills(
    config_path: &Path,
    full_config: &Config,
    config_dir: &Path,
    pending_skill_ids: &[String],
    concurrency: usize,
    ui: &UiContext,
    emit_human_output: bool,
) -> Result<ApplyOutcome, EdenError> {
    let selected_ids = pending_skill_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let selected_skills = full_config
        .skills
        .iter()
        .filter(|skill| selected_ids.contains(skill.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if selected_skills.is_empty() {
        return Ok(ApplyOutcome::default());
    }

    let selected_config = Config {
        version: full_config.version,
        storage_root: full_config.storage_root.clone(),
        reactor: full_config.reactor,
        skills: selected_skills,
    };

    materialize_fetch_heads(&selected_config, config_dir)?;

    let reactor = SkillReactor::new(concurrency).map_err(EdenError::from)?;
    let sync_summary = sync_sources_async_with_reactor(&selected_config, config_dir, reactor)
        .await
        .map_err(EdenError::from)?;
    if emit_human_output {
        print_source_sync_summary_human(ui, &sync_summary);
    }
    if let Some(err) = source_sync_failure_error(&sync_summary) {
        return Err(err);
    }

    let safety_reports = analyze_skills(&selected_config, config_dir)?;
    persist_reports(&safety_reports)?;
    if emit_human_output {
        print_safety_summary_human(ui, &safety_reports);
    }

    let no_exec_skill_ids = no_exec_skill_ids(&safety_reports);
    let plan = build_plan(&selected_config, config_dir)?;
    let mut applied_targets = Vec::new();
    let mut skipped_skill_ids = Vec::new();
    for item in &plan {
        match item.action {
            Action::Create | Action::Update => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    push_skipped_skill(&mut skipped_skill_ids, &item.skill_id);
                    continue;
                }
                apply_plan_item(item)?;
                applied_targets.push(AppliedInstallTargetLine {
                    skill_id: item.skill_id.clone(),
                    target_path: item.target_path.clone(),
                    mode: item.install_mode.as_str().to_string(),
                });
            }
            Action::Conflict => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    push_skipped_skill(&mut skipped_skill_ids, &item.skill_id);
                }
            }
            Action::Noop | Action::Remove => {}
        }
    }
    if emit_human_output {
        print_install_result_lines(ui, &applied_targets, &skipped_skill_ids);
    }

    let verify_issues = verify_config_state(&selected_config, config_dir)?;
    if !verify_issues.is_empty() {
        return Err(EdenError::Runtime(format!(
            "post-update verification failed with {} issue(s); first: [{}] {} {}",
            verify_issues.len(),
            verify_issues[0].check,
            verify_issues[0].skill_id,
            verify_issues[0].message
        )));
    }

    let resolved_commits = collect_resolved_commits(full_config, config_dir);
    write_lock_for_config_with_commits(config_path, full_config, config_dir, &resolved_commits)?;
    let applied_skill_ids = selected_config
        .skills
        .iter()
        .map(|skill| skill.id.clone())
        .collect::<HashSet<_>>();
    Ok(ApplyOutcome { applied_skill_ids })
}

fn no_exec_skill_ids(reports: &[SkillSafetyReport]) -> HashSet<&str> {
    reports
        .iter()
        .filter(|report| report.no_exec_metadata_only)
        .map(|report| report.skill_id.as_str())
        .collect()
}

fn collect_resolved_commits(config: &Config, config_dir: &Path) -> HashMap<String, String> {
    let storage_root = match resolve_path_string(&config.storage_root, config_dir) {
        Ok(path) => path,
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

fn materialize_fetch_heads(config: &Config, config_dir: &Path) -> Result<(), EdenError> {
    let storage_root = resolve_path_string(&config.storage_root, config_dir)?;
    for skill in &config.skills {
        let repo_dir = resolve_skill_storage_root(&storage_root, skill);
        if !repo_dir.join(".git").exists() {
            continue;
        }
        let Some(fetch_head) = read_fetch_head_sha(&repo_dir) else {
            continue;
        };
        if read_head_sha(&repo_dir).as_deref() == Some(fetch_head.as_str()) {
            continue;
        }
        run_git_command(
            Command::new("git")
                .arg("-C")
                .arg(&repo_dir)
                .arg("reset")
                .arg("--hard")
                .arg("FETCH_HEAD"),
            &format!("apply fetched commit for `{}`", skill.id),
        )
        .map_err(EdenError::Runtime)?;
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

fn style_skill_id(ui: &UiContext, skill_id: &str) -> String {
    ui.styled_skill_id(skill_id)
}

fn style_mode_label(ui: &UiContext, mode: &str) -> String {
    let raw = format!("({mode})");
    ui.styled_secondary(&raw)
}

fn style_tree_connector(ui: &UiContext, connector: &str) -> String {
    if ui.colors_enabled() {
        connector.dimmed().to_string()
    } else {
        connector.to_string()
    }
}

fn registry_status_cell(ui: &UiContext, status: &RegistrySyncStatus) -> String {
    ui.styled_status(status.as_str())
}

fn skill_refresh_status_cell(ui: &UiContext, status: SkillRefreshStatus) -> String {
    ui.styled_status(status.table_label())
}

fn style_detail_cell(ui: &UiContext, detail: Option<&str>) -> String {
    detail.map_or_else(String::new, |detail| ui.styled_secondary(detail))
}

async fn sync_registry_task(
    task: RegistrySyncTask,
    reactor: SkillReactor,
) -> Result<RegistrySyncResult, RegistrySyncResult> {
    let failed_name = task.name.clone();
    let failed_url = task.url.clone();

    let task_label = format!("sync registry `{}`", task.name);
    match reactor
        .run_blocking(&task_label, move || sync_registry_task_blocking(task))
        .await
    {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(result)) => Err(result),
        Err(err) => Err(RegistrySyncResult {
            name: failed_name,
            status: RegistrySyncStatus::Failed,
            url: failed_url,
            detail: Some(err.to_string()),
        }),
    }
}

fn sync_registry_task_blocking(
    task: RegistrySyncTask,
) -> Result<Result<RegistrySyncResult, RegistrySyncResult>, EdenError> {
    let failed = |detail: String| RegistrySyncResult {
        name: task.name.clone(),
        status: RegistrySyncStatus::Failed,
        url: task.url.clone(),
        detail: Some(detail),
    };

    if let Some(parent) = task.local_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    let git_dir = task.local_dir.join(".git");
    if !git_dir.exists() {
        let clone_result = run_git_command(
            Command::new("git")
                .arg("clone")
                .arg("--depth")
                .arg("1")
                .arg(&task.url)
                .arg(&task.local_dir),
            &format!("clone registry `{}`", task.name),
        );
        return Ok(match clone_result {
            Ok(_) => {
                write_registry_sync_marker(&task.local_dir)?;
                Ok(RegistrySyncResult {
                    name: task.name,
                    status: RegistrySyncStatus::Cloned,
                    url: task.url,
                    detail: None,
                })
            }
            Err(detail) => Err(failed(detail)),
        });
    }

    let head_before = read_head_sha(&task.local_dir);
    let fetch_result = run_git_command(
        Command::new("git")
            .arg("-C")
            .arg(&task.local_dir)
            .arg("fetch")
            .arg("--depth")
            .arg("1")
            .arg("origin"),
        &format!("fetch registry `{}`", task.name),
    );
    if let Err(detail) = fetch_result {
        return Ok(Err(failed(detail)));
    }

    let reset_result = run_git_command(
        Command::new("git")
            .arg("-C")
            .arg(&task.local_dir)
            .arg("reset")
            .arg("--hard")
            .arg("FETCH_HEAD"),
        &format!("reset registry `{}`", task.name),
    );
    if let Err(detail) = reset_result {
        return Ok(Err(failed(detail)));
    }

    let head_after = read_head_sha(&task.local_dir);
    let status = if head_before.is_some() && head_before == head_after {
        RegistrySyncStatus::Skipped
    } else {
        RegistrySyncStatus::Updated
    };
    write_registry_sync_marker(&task.local_dir)?;
    Ok(Ok(RegistrySyncResult {
        name: task.name,
        status,
        url: task.url,
        detail: None,
    }))
}

fn write_registry_sync_marker(registry_dir: &std::path::Path) -> Result<(), EdenError> {
    let now_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| EdenError::Runtime(format!("failed to get system time: {err}")))?
        .as_secs()
        .to_string();
    fs::write(registry_dir.join(REGISTRY_SYNC_MARKER_FILE), now_epoch)?;
    Ok(())
}
