//! Skill installation from URLs, local paths, and registries.
//!
//! Dispatches to one of three install modes based on source format
//! detection: registry name lookup, remote URL (GitHub / SSH / HTTPS),
//! or local directory path. Each mode handles discovery, user selection,
//! config mutation, source sync, and lock file updates.

mod discovery;
mod dry_run;
mod execute;
mod output;
mod platform;

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use eden_skills_core::agents::detect_installed_agent_targets;
use eden_skills_core::config::{config_dir_from_path, validate_config, Config};
use eden_skills_core::config::{SkillConfig, TargetConfig};
use eden_skills_core::discovery::{discover_skills, DiscoveredSkill};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::{normalize_lexical, resolve_path_string};
use eden_skills_core::source::sync_sources_async;
use eden_skills_core::source_format::{
    derive_skill_id_from_source_repo, detect_install_source, DetectedInstallSource,
    UrlInstallSource,
};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use crate::ui::{StatusSymbol, UiContext};
use crate::DEFAULT_CONFIG_PATH;

use super::common::{
    agent_kind_label, ensure_git_available, load_config_with_context, parse_target_specs,
    print_source_sync_step_summary_human, print_warning, resolve_config_path,
    resolve_registry_mode_skills_for_execution, source_sync_failure_error, write_lock_for_config,
    write_normalized_config,
};
use super::config_ops::default_config_template;
use super::InstallRequest;

use self::adapter::DockerAdapter;
use self::discovery::{
    discover_remote_skills_via_temp_clone, join_scoped_subpath, print_discovery_json,
    print_discovery_preview, resolve_local_install_selection,
    seed_repo_cache_from_discovery_checkout,
};
use self::dry_run::print_install_dry_run;
use self::execute::{
    default_docker_install_target, default_install_target, execute_install_plan_async,
    install_local_source_skill_async, print_install_success_json, should_preserve_existing_targets,
    upsert_mode_a_skill, upsert_mode_b_skill, InstallExecutionSummary,
};
use self::output::{
    print_docker_cp_hints, print_install_result_lines, print_install_result_summary,
};
use self::platform::{
    requested_install_mode, resolve_default_install_mode_decision,
    warn_windows_hardcopy_fallback_if_needed,
};

mod adapter {
    pub(super) use eden_skills_core::adapter::DockerAdapter;
}

struct StepProgress {
    progress_bar: Option<ProgressBar>,
    label: String,
    total_steps: usize,
    synced: usize,
    failed: usize,
    emit_test_step_lines: bool,
}

impl StepProgress {
    fn new(ui: &UiContext, label: &str, total_steps: usize) -> Self {
        let progress_bar = if ui.spinner_enabled() && total_steps > 0 {
            let bar = ProgressBar::with_draw_target(
                Some(total_steps as u64),
                ProgressDrawTarget::stderr(),
            );
            let style = ProgressStyle::with_template("  {prefix} [{pos}/{len}] {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_bar());
            bar.set_style(style);
            bar.set_prefix(ui.action_prefix(label));
            Some(bar)
        } else {
            None
        };

        Self {
            progress_bar,
            label: label.to_string(),
            total_steps,
            synced: 0,
            failed: 0,
            emit_test_step_lines: std::env::var("EDEN_SKILLS_FORCE_TTY")
                .ok()
                .is_some_and(|value| value == "1"),
        }
    }

    fn start_step(&self, skill_id: &str) {
        if let Some(bar) = &self.progress_bar {
            bar.set_message(format!("{skill_id}…"));
        }
    }

    fn record_step(
        &mut self,
        ui: &UiContext,
        zero_based_step: usize,
        skill_id: &str,
        failed: bool,
    ) {
        if failed {
            self.failed += 1;
        } else {
            self.synced += 1;
        }
        if self.emit_test_step_lines {
            println!(
                "  {} [{}/{}] {}…",
                ui.action_prefix(&self.label),
                zero_based_step + 1,
                self.total_steps,
                skill_id
            );
        }
        if let Some(bar) = &self.progress_bar {
            bar.set_position((zero_based_step + 1) as u64);
        }
    }

    fn finish_with_sync_summary(self, ui: &UiContext) {
        if let Some(bar) = self.progress_bar {
            bar.finish_and_clear();
        }
        print_source_sync_step_summary_human(ui, self.synced, self.failed);
    }

    fn finish_quiet(self) {
        if let Some(bar) = self.progress_bar {
            bar.finish_and_clear();
        }
    }
}

/// Install skills from a URL, local path, or registry name.
///
/// Detects the source format, discovers available skills, applies user
/// selections, syncs sources, writes config and lock, and installs
/// targets via the appropriate adapter.
///
/// # Errors
///
/// Returns [`EdenError`] on config I/O failures, invalid source format,
/// git clone failures, adapter errors, or user cancellation.
pub async fn install_async(req: InstallRequest) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(&req.config_path)?;
    let config_path = config_path_buf.as_path();
    let default_config_path = resolve_config_path(DEFAULT_CONFIG_PATH)?;
    let auto_create_missing_parent = config_path == default_config_path.as_path();
    let cwd = std::env::current_dir().map_err(EdenError::Io)?;
    let detected_source = detect_install_source(&req.source, &cwd)?;
    let ui = UiContext::from_env(req.options.json);
    if !req.list {
        warn_windows_hardcopy_fallback_if_needed(&ui, resolve_default_install_mode_decision());
    }
    match detected_source {
        DetectedInstallSource::RegistryName(skill_name) => {
            ensure_install_config_exists(config_path, &ui, auto_create_missing_parent)?;
            install_registry_mode_async(&req, config_path, &skill_name, &ui).await
        }
        DetectedInstallSource::Url(url_source) => {
            if !req.list || req.dry_run {
                ensure_install_config_exists(config_path, &ui, auto_create_missing_parent)?;
            }
            install_url_mode_async(&req, config_path, &url_source, &ui).await
        }
    }
}

fn ensure_install_config_exists(
    config_path: &Path,
    ui: &UiContext,
    auto_create_missing_parent: bool,
) -> Result<(), EdenError> {
    if config_path.exists() {
        return Ok(());
    }

    let parent = config_path.parent().unwrap_or(Path::new("."));
    if !parent.exists() {
        if auto_create_missing_parent {
            fs::create_dir_all(parent)?;
        } else {
            return Err(EdenError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "config parent directory does not exist: {}",
                    parent.display()
                ),
            )));
        }
    }

    fs::write(config_path, default_config_template())?;
    if !ui.json_mode() {
        let display_path = ui.styled_path(&config_path.display().to_string());
        if ui.symbols_enabled() {
            println!(
                "{} Created config at {}",
                ui.status_symbol(StatusSymbol::Success),
                display_path
            );
        } else {
            println!("Created config at {display_path}");
        }
    }
    Ok(())
}

async fn install_registry_mode_async(
    req: &InstallRequest,
    config_path: &Path,
    skill_name: &str,
    ui: &UiContext,
) -> Result<(), EdenError> {
    if req.id.is_some() || req.r#ref.is_some() {
        return Err(EdenError::InvalidArguments(
            "--id/--ref are only supported for URL-mode install sources".to_string(),
        ));
    }

    let loaded = load_config_with_context(config_path, req.options.strict)?;
    for warning in loaded.warnings {
        print_warning(ui, &warning);
    }
    let config_dir = config_dir_from_path(config_path);

    let mut config = loaded.config;
    let requested_constraint = req.version.clone().unwrap_or_else(|| "*".to_string());
    let target_override = resolve_registry_mode_install_targets(&req.target, ui).await?;
    upsert_mode_b_skill(
        &mut config,
        skill_name,
        &requested_constraint,
        req.registry.as_deref(),
        target_override,
        requested_install_mode(req.copy),
    )?;
    validate_config(&config, &config_dir)?;

    let mut single_skill_config = select_single_skill_config(&config, skill_name)?;

    single_skill_config = resolve_registry_mode_skills_for_execution(
        config_path,
        &single_skill_config,
        &config_dir,
        ui,
    )?;

    if req.dry_run {
        let preview_skill_ids = vec![skill_name.to_string()];
        print_install_dry_run(
            ui,
            req.options.json,
            &single_skill_config,
            &preview_skill_ids,
            &config_dir,
            true,
        )?;
        return Ok(());
    }

    write_normalized_config(config_path, &config)?;

    ensure_git_available()?;
    let mut sync_progress = StepProgress::new(ui, "Syncing", 1);
    sync_progress.start_step(skill_name);
    let sync_summary = sync_sources_async(&single_skill_config, &config_dir).await?;
    sync_progress.record_step(ui, 0, skill_name, sync_summary.failed > 0);
    if !req.options.json {
        sync_progress.finish_with_sync_summary(ui);
    }
    if let Some(err) = source_sync_failure_error(&sync_summary) {
        return Err(err);
    }

    let mut execution_summary = execute_install_plan_async(
        &single_skill_config,
        &config_dir,
        req.options.strict,
        req.force,
        ui,
    )
    .await?;

    let full_loaded = load_config_with_context(config_path, false)?;
    write_lock_for_config(config_path, &full_loaded.config, &config_dir)?;

    if execution_summary.installed_targets.is_empty() && execution_summary.conflicts == 0 {
        execution_summary.skipped_skills += 1;
    }

    if req.options.json {
        print_install_success_json(skill_name, &requested_constraint)?;
    } else {
        print_install_result_lines(ui, &execution_summary.installed_targets);
        print_install_result_summary(
            ui,
            &execution_summary,
            unique_agent_count(&single_skill_config),
        );
        print_docker_cp_hints(ui, &execution_summary.docker_cp_hint_containers);
    }
    Ok(())
}

async fn install_url_mode_async(
    req: &InstallRequest,
    config_path: &Path,
    url_source: &UrlInstallSource,
    ui: &UiContext,
) -> Result<(), EdenError> {
    if url_source.is_local {
        return install_local_url_mode_async(req, config_path, url_source, ui).await;
    }

    install_remote_url_mode_async(req, config_path, url_source, ui).await
}

async fn install_remote_url_mode_async(
    req: &InstallRequest,
    config_path: &Path,
    url_source: &UrlInstallSource,
    ui: &UiContext,
) -> Result<(), EdenError> {
    ensure_git_available()?;
    let source_ref = req
        .r#ref
        .clone()
        .or_else(|| url_source.reference.clone())
        .unwrap_or_else(|| "main".to_string());
    let scope_subpath = url_source
        .subpath
        .clone()
        .unwrap_or_else(|| ".".to_string());
    let clone_spinner = ui.spinner(
        "Cloning",
        format!("{}@{} ({})", url_source.repo, source_ref, scope_subpath),
    );
    let mut remote_discovery =
        match discover_remote_skills_via_temp_clone(&url_source.repo, &source_ref, &scope_subpath)
            .await
        {
            Ok(discovery) => {
                clone_spinner.finish_success(ui);
                discovery
            }
            Err(err) => {
                clone_spinner.finish_failure(ui, &err.to_string());
                return Err(err);
            }
        };
    if remote_discovery.discovered.is_empty() {
        print_warning(ui, "No SKILL.md found; installing directory as-is.");
    }

    if req.list && !req.dry_run {
        if req.options.json {
            print_discovery_json(&remote_discovery.discovered)?;
        } else {
            print_discovery_preview(ui, &remote_discovery.discovered, true);
        }
        return Ok(());
    }

    if remote_discovery.discovered.is_empty() {
        if !req.all && !req.skill.is_empty() {
            return Err(EdenError::InvalidArguments(format!(
                "unknown skill name(s): {}; available: (none discovered)",
                req.skill.join(", ")
            )));
        }
        let fallback_name = req
            .id
            .clone()
            .map(Ok)
            .unwrap_or_else(|| derive_skill_id_from_source_repo(&url_source.repo))?;
        remote_discovery.discovered.push(DiscoveredSkill {
            name: fallback_name,
            description: String::new(),
            subpath: ".".to_string(),
        });
    }

    let selected = resolve_local_install_selection(
        &remote_discovery.discovered,
        req.all,
        &req.skill,
        req.yes,
        ui,
    )?;
    if selected.is_empty() {
        return Ok(());
    }
    let loaded = load_config_with_context(config_path, req.options.strict)?;
    for warning in loaded.warnings {
        print_warning(ui, &warning);
    }
    let config_dir = config_dir_from_path(config_path);
    let mut config = loaded.config;

    let mut selected_ids = Vec::new();
    for skill in &selected {
        let skill_id = if selected.len() == 1 {
            req.id.clone().unwrap_or_else(|| skill.name.clone())
        } else {
            skill.name.clone()
        };
        let effective_subpath = join_scoped_subpath(&scope_subpath, &skill.subpath);
        let existing_skill = config
            .skills
            .iter()
            .find(|existing| existing.id == skill_id)
            .cloned();
        let target_override =
            resolve_url_mode_install_targets(&req.target, existing_skill.as_ref(), ui).await?;
        upsert_mode_a_skill(
            &mut config,
            &skill_id,
            &url_source.repo,
            &effective_subpath,
            &source_ref,
            target_override,
            requested_install_mode(req.copy),
        )?;
        selected_ids.push(skill_id);
    }

    validate_config(&config, &config_dir)?;
    let selected_config = select_config_skills(&config, &selected_ids);

    if req.dry_run {
        print_install_dry_run(
            ui,
            req.options.json,
            &selected_config,
            &selected_ids,
            &config_dir,
            req.list,
        )?;
        return Ok(());
    }

    write_normalized_config(config_path, &config)?;
    let storage_root = resolve_path_string(&selected_config.storage_root, &config_dir)?;
    seed_repo_cache_from_discovery_checkout(
        remote_discovery.temp_checkout.take(),
        &storage_root,
        &url_source.repo,
        &source_ref,
    )?;

    let sync_spinner = ui.spinner("Syncing", format!("{} skill sources…", selected_ids.len()));
    let sync_summary = sync_sources_async(&selected_config, &config_dir).await?;
    if sync_summary.failed > 0 {
        sync_spinner.finish_failure(ui, &format!("{} source(s) failed", sync_summary.failed));
    } else {
        sync_spinner.finish_success(ui);
    }
    if !req.options.json {
        print_source_sync_step_summary_human(
            ui,
            sync_summary.cloned + sync_summary.updated + sync_summary.skipped,
            sync_summary.failed,
        );
    }
    if let Some(err) = source_sync_failure_error(&sync_summary) {
        return Err(err);
    }

    let mut execution_summary = InstallExecutionSummary::default();
    let mut install_progress = StepProgress::new(ui, "Installing", selected_ids.len());
    for (index, skill_id) in selected_ids.iter().enumerate() {
        install_progress.start_step(skill_id);
        let single_skill_config = select_single_skill_config(&selected_config, skill_id)?;
        let skill_summary = execute_install_plan_async(
            &single_skill_config,
            &config_dir,
            req.options.strict,
            req.force,
            ui,
        )
        .await?;
        let skill_was_noop =
            skill_summary.installed_targets.is_empty() && skill_summary.conflicts == 0;
        if skill_was_noop {
            execution_summary.skipped_skills += 1;
        }
        install_progress.record_step(ui, index, skill_id, false);
        execution_summary.merge(skill_summary);
    }
    if !req.options.json {
        install_progress.finish_quiet();
    }

    let full_loaded = load_config_with_context(config_path, false)?;
    write_lock_for_config(config_path, &full_loaded.config, &config_dir)?;

    if req.options.json {
        let payload = serde_json::json!({
            "skills": selected_ids,
            "status": "installed",
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to encode install json: {err}")))?;
        println!("{encoded}");
    } else {
        print_install_result_lines(ui, &execution_summary.installed_targets);
        print_install_result_summary(ui, &execution_summary, unique_agent_count(&selected_config));
        print_docker_cp_hints(ui, &execution_summary.docker_cp_hint_containers);
    }

    Ok(())
}

async fn install_local_url_mode_async(
    req: &InstallRequest,
    config_path: &Path,
    url_source: &UrlInstallSource,
    ui: &UiContext,
) -> Result<(), EdenError> {
    let config_dir = config_dir_from_path(config_path);
    let source_root = resolve_path_string(&url_source.repo, &config_dir)?;
    let scope_subpath = url_source
        .subpath
        .clone()
        .unwrap_or_else(|| ".".to_string());
    let discovery_root = normalize_lexical(&source_root.join(&scope_subpath));
    if !discovery_root.exists() {
        return Err(EdenError::Runtime(format!(
            "discovery path does not exist: {}",
            discovery_root.display()
        )));
    }

    let mut discovered = discover_skills(&discovery_root)?;
    if discovered.is_empty() {
        print_warning(ui, "No SKILL.md found; installing directory as-is.");
    }

    if req.list && !req.dry_run {
        if req.options.json {
            print_discovery_json(&discovered)?;
        } else {
            print_discovery_preview(ui, &discovered, true);
        }
        return Ok(());
    }

    if discovered.is_empty() {
        if !req.all && !req.skill.is_empty() {
            return Err(EdenError::InvalidArguments(format!(
                "unknown skill name(s): {}; available: (none discovered)",
                req.skill.join(", ")
            )));
        }
        let fallback_name = req
            .id
            .clone()
            .map(Ok)
            .unwrap_or_else(|| derive_skill_id_from_source_repo(&url_source.repo))?;
        discovered.push(DiscoveredSkill {
            name: fallback_name,
            description: String::new(),
            subpath: ".".to_string(),
        });
    }

    let selected = resolve_local_install_selection(&discovered, req.all, &req.skill, req.yes, ui)?;
    if selected.is_empty() {
        return Ok(());
    }
    let loaded = load_config_with_context(config_path, req.options.strict)?;
    for warning in loaded.warnings {
        print_warning(ui, &warning);
    }
    let mut config = loaded.config;
    let source_ref = req
        .r#ref
        .clone()
        .or_else(|| url_source.reference.clone())
        .unwrap_or_else(|| "main".to_string());

    let mut selected_ids = Vec::new();
    for skill in &selected {
        let skill_id = if selected.len() == 1 {
            req.id.clone().unwrap_or_else(|| skill.name.clone())
        } else {
            skill.name.clone()
        };
        let effective_subpath = join_scoped_subpath(&scope_subpath, &skill.subpath);
        let existing_skill = config
            .skills
            .iter()
            .find(|existing| existing.id == skill_id)
            .cloned();
        let target_override =
            resolve_url_mode_install_targets(&req.target, existing_skill.as_ref(), ui).await?;
        upsert_mode_a_skill(
            &mut config,
            &skill_id,
            &url_source.repo,
            &effective_subpath,
            &source_ref,
            target_override,
            requested_install_mode(req.copy),
        )?;
        selected_ids.push(skill_id);
    }

    validate_config(&config, &config_dir)?;
    let selected_config = select_config_skills(&config, &selected_ids);

    if req.dry_run {
        print_install_dry_run(
            ui,
            req.options.json,
            &selected_config,
            &selected_ids,
            &config_dir,
            req.list,
        )?;
        return Ok(());
    }

    write_normalized_config(config_path, &config)?;

    let mut execution_summary = InstallExecutionSummary::default();
    for skill_id in &selected_ids {
        let single = select_single_skill_config(&selected_config, skill_id)?;
        let skill_summary = install_local_source_skill_async(
            &single,
            &config_dir,
            req.options.strict,
            req.force,
            ui,
        )
        .await?;
        let skill_was_noop =
            skill_summary.installed_targets.is_empty() && skill_summary.conflicts == 0;
        if skill_was_noop {
            execution_summary.skipped_skills += 1;
        }
        execution_summary.merge(skill_summary);
    }

    let full_loaded = load_config_with_context(config_path, false)?;
    write_lock_for_config(config_path, &full_loaded.config, &config_dir)?;

    if req.options.json {
        let payload = serde_json::json!({
            "skills": selected_ids,
            "status": "installed",
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to encode install json: {err}")))?;
        println!("{encoded}");
    } else {
        print_install_result_lines(ui, &execution_summary.installed_targets);
        print_install_result_summary(ui, &execution_summary, unique_agent_count(&selected_config));
        print_docker_cp_hints(ui, &execution_summary.docker_cp_hint_containers);
    }

    Ok(())
}

fn select_config_skills(config: &Config, skill_ids: &[String]) -> Config {
    Config {
        version: config.version,
        storage_root: config.storage_root.clone(),
        reactor: config.reactor,
        skills: config
            .skills
            .iter()
            .filter(|skill| skill_ids.iter().any(|id| id == &skill.id))
            .cloned()
            .collect(),
    }
}

fn select_single_skill_config(config: &Config, skill_id: &str) -> Result<Config, EdenError> {
    let single = Config {
        version: config.version,
        storage_root: config.storage_root.clone(),
        reactor: config.reactor,
        skills: config
            .skills
            .iter()
            .find(|skill| skill.id == skill_id)
            .cloned()
            .into_iter()
            .collect(),
    };
    if single.skills.is_empty() {
        return Err(EdenError::Runtime(format!(
            "failed to select installed skill `{skill_id}`"
        )));
    }
    Ok(single)
}

fn unique_agent_count(config: &Config) -> usize {
    let mut agents = HashSet::new();
    for skill in &config.skills {
        for target in &skill.targets {
            agents.insert(agent_kind_label(&target.agent));
        }
    }
    agents.len()
}

async fn resolve_registry_mode_install_targets(
    target_specs: &[String],
    ui: &UiContext,
) -> Result<Option<Vec<TargetConfig>>, EdenError> {
    match target_specs {
        [] => Ok(None),
        [spec] => Ok(Some(resolve_install_special_target_spec(spec, ui).await?)),
        _ => Err(EdenError::InvalidArguments(
            "registry-mode install accepts at most one --target (`local` or `docker:<container>`)"
                .to_string(),
        )),
    }
}

async fn resolve_url_mode_install_targets(
    target_specs: &[String],
    existing_skill: Option<&SkillConfig>,
    ui: &UiContext,
) -> Result<Option<Vec<TargetConfig>>, EdenError> {
    if !target_specs.is_empty() {
        return resolve_explicit_install_targets(target_specs, ui).await;
    }

    if existing_skill.is_some_and(should_preserve_existing_targets) {
        return Ok(None);
    }

    let detected = detect_installed_agent_targets()?;
    if !detected.is_empty() {
        return Ok(Some(detected));
    }

    print_warning(
        ui,
        "No installed agents detected; defaulting to claude-code (~/.claude/skills/)",
    );
    Ok(Some(vec![default_install_target()]))
}

async fn resolve_explicit_install_targets(
    target_specs: &[String],
    ui: &UiContext,
) -> Result<Option<Vec<TargetConfig>>, EdenError> {
    match target_specs {
        [spec] if spec == "local" || spec.starts_with("docker:") => {
            Ok(Some(resolve_install_special_target_spec(spec, ui).await?))
        }
        specs if specs
            .iter()
            .any(|spec| spec == "local" || spec.starts_with("docker:")) =>
        {
            Err(EdenError::InvalidArguments(
                "`--target local` and `--target docker:<container>` cannot be combined with other target specs"
                    .to_string(),
            ))
        }
        _ => parse_target_specs(target_specs).map(Some),
    }
}

async fn resolve_install_special_target_spec(
    spec: &str,
    ui: &UiContext,
) -> Result<Vec<TargetConfig>, EdenError> {
    if spec == "local" {
        return Ok(vec![default_install_target()]);
    }
    if let Some(container_name) = spec.strip_prefix("docker:") {
        if container_name.trim().is_empty() {
            return Err(EdenError::InvalidArguments(
                "invalid --target `docker:`: container name is required".to_string(),
            ));
        }
        return detect_docker_install_targets(container_name, ui).await;
    }
    Err(EdenError::InvalidArguments(format!(
        "invalid --target `{spec}` (expected `local` or `docker:<container>`)"
    )))
}

async fn detect_docker_install_targets(
    container_name: &str,
    ui: &UiContext,
) -> Result<Vec<TargetConfig>, EdenError> {
    let adapter = DockerAdapter::new(container_name).map_err(EdenError::from)?;
    let detected = adapter.detect_agents().await.map_err(EdenError::from)?;
    if !detected.is_empty() {
        return Ok(detected);
    }
    print_warning(
        ui,
        &format!(
            "No installed agents detected in container '{container_name}'; defaulting to claude-code."
        ),
    );
    Ok(vec![default_docker_install_target(container_name)])
}
