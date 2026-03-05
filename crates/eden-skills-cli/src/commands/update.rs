//! Registry refresh via the `update` command.
//!
//! Resolves configured registries, clones or fetches each in parallel
//! using the reactor, and records sync markers. Results are rendered as
//! a table with per-registry status and a timing footer.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use comfy_table::{ColumnConstraint, Width};
use eden_skills_core::config::config_dir_from_path;
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::resolve_path_string;
use eden_skills_core::reactor::SkillReactor;
use eden_skills_core::registry::{parse_registry_specs_from_toml, sort_registry_specs_by_priority};

use super::common::{
    ensure_git_available, load_config_with_context, read_head_sha, resolve_config_path,
    resolve_effective_reactor_concurrency, run_git_command, REGISTRY_SYNC_MARKER_FILE,
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
    if registry_specs.is_empty() {
        super::common::print_warning(&ui, "no registries configured; skipping update");
        return Ok(());
    }
    ensure_git_available()?;

    let concurrency = resolve_effective_reactor_concurrency(
        req.concurrency,
        loaded.config.reactor.concurrency,
        "update.concurrency",
    )?;

    let config_dir = config_dir_from_path(config_path);
    let storage_root = resolve_path_string(&loaded.config.storage_root, &config_dir)?;
    let registries_root = storage_root.join("registries");
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
    let started = Instant::now();
    let outcomes = reactor
        .run_phase_a(tasks, move |task| {
            let reactor = reactor;
            async move { sync_registry_task(task, reactor).await }
        })
        .await
        .map_err(EdenError::from)?;
    let elapsed_ms = started.elapsed().as_millis() as u64;

    let mut results = Vec::new();
    for outcome in outcomes {
        match outcome.result {
            Ok(result) | Err(result) => results.push(result),
        }
    }
    results.sort_by(|left, right| left.name.cmp(&right.name));

    let failed_count = results
        .iter()
        .filter(|result| matches!(result.status, RegistrySyncStatus::Failed))
        .count();

    if req.options.json {
        let payload = serde_json::json!({
            "registries": results.iter().map(|result| {
                serde_json::json!({
                    "name": result.name,
                    "status": result.status.as_str(),
                    "url": result.url,
                    "detail": result.detail,
                })
            }).collect::<Vec<_>>(),
            "failed": failed_count,
            "elapsed_ms": elapsed_ms,
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to encode update json: {err}")))?;
        println!("{encoded}");
    } else {
        println!(
            "{}  {} registries synced",
            ui.action_prefix("Update"),
            results.len()
        );
        println!();

        let mut table = ui.table(&["Registry", "Status", "Detail"]);
        if let Some(column) = table.column_mut(1) {
            column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(10)));
        }
        for result in &results {
            table.add_row(vec![
                result.name.clone(),
                registry_status_cell(&result.status),
                result.detail.clone().unwrap_or_default(),
            ]);
        }
        println!("{table}");

        let elapsed_seconds = elapsed_ms as f64 / 1000.0;
        let summary_symbol = if failed_count == 0 {
            StatusSymbol::Success
        } else {
            StatusSymbol::Failure
        };
        println!();
        println!(
            "  {} {} failed [{elapsed_seconds:.1}s]",
            ui.status_symbol(summary_symbol),
            failed_count
        );
        for result in results
            .iter()
            .filter(|result| matches!(result.status, RegistrySyncStatus::Failed))
        {
            if let Some(detail) = &result.detail {
                super::common::print_warning(
                    &ui,
                    &format!("registry `{}` failed: {detail}", result.name),
                );
            }
        }
    }

    Ok(())
}

fn registry_status_cell(status: &RegistrySyncStatus) -> String {
    status.as_str().to_string()
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
