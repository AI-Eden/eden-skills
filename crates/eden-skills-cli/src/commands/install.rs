//! Skill installation from URLs, local paths, and registries.
//!
//! Dispatches to one of three install modes based on source format
//! detection: registry name lookup, remote URL (GitHub / SSH / HTTPS),
//! or local directory path. Each mode handles discovery, user selection,
//! config mutation, source sync, and lock file updates.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use dialoguer::{Confirm, Input};
use eden_skills_core::agents::detect_installed_agent_targets;
use eden_skills_core::config::{
    config_dir_from_path, default_verify_checks_for_mode, encode_registry_mode_repo,
    validate_config, InstallMode, SourceConfig,
};
use eden_skills_core::config::{AgentKind, Config, SkillConfig, TargetConfig};
use eden_skills_core::discovery::{discover_skills, DiscoveredSkill};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::{normalize_lexical, resolve_path_string, resolve_target_path};
use eden_skills_core::plan::{build_plan, Action};
use eden_skills_core::source::sync_sources_async;
use eden_skills_core::source_format::{
    derive_skill_id_from_source_repo, detect_install_source, DetectedInstallSource,
    UrlInstallSource,
};

use crate::ui::{abbreviate_home_path, abbreviate_repo_url, StatusSymbol, UiContext};
use crate::DEFAULT_CONFIG_PATH;

use super::common::{
    agent_kind_label, apply_plan_item, copy_recursively, ensure_git_available, ensure_parent_dir,
    load_config_with_context, parse_target_specs, print_source_sync_summary, print_warning,
    remove_path, resolve_config_path, resolve_registry_mode_skills_for_execution, run_git_command,
    source_sync_failure_error, write_lock_for_config, write_normalized_config,
};
use super::config_ops::default_config_template;
use super::InstallRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DefaultInstallModeDecision {
    mode: InstallMode,
    warn_windows_hardcopy_fallback: bool,
}

#[derive(Debug, Default)]
struct InstallExecutionSummary {
    installed_targets: Vec<InstallTargetLine>,
    conflicts: usize,
}

#[derive(Debug)]
struct InstallTargetLine {
    skill_id: String,
    target_path: String,
    mode: String,
}

impl InstallExecutionSummary {
    fn merge(&mut self, mut other: InstallExecutionSummary) {
        self.conflicts += other.conflicts;
        self.installed_targets.append(&mut other.installed_targets);
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
            if !req.list {
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
        let display_path = crate::ui::abbreviate_home_path(&config_path.display().to_string());
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
    upsert_mode_b_skill(
        &mut config,
        skill_name,
        &requested_constraint,
        req.registry.as_deref(),
        &req.target,
        requested_install_mode(req.copy),
    )?;
    validate_config(&config, &config_dir)?;

    let mut single_skill_config = select_single_skill_config(&config, skill_name)?;

    single_skill_config =
        resolve_registry_mode_skills_for_execution(config_path, &single_skill_config, &config_dir)?;

    if req.dry_run {
        print_install_dry_run(
            ui,
            req.options.json,
            &single_skill_config,
            skill_name,
            &requested_constraint,
            &config_dir,
        )?;
        return Ok(());
    }

    write_normalized_config(config_path, &config)?;

    ensure_git_available()?;
    let sync_summary = sync_sources_async(&single_skill_config, &config_dir).await?;
    if !req.options.json {
        print_source_sync_summary(&sync_summary);
    }
    if let Some(err) = source_sync_failure_error(&sync_summary) {
        return Err(err);
    }

    let execution_summary =
        execute_install_plan(&single_skill_config, &config_dir, req.options.strict)?;

    let full_loaded = load_config_with_context(config_path, false)?;
    write_lock_for_config(config_path, &full_loaded.config, &config_dir)?;

    if req.options.json {
        print_install_success_json(skill_name, &requested_constraint)?;
    } else {
        print_install_result_lines(ui, &execution_summary.installed_targets);
        print_install_result_summary(
            ui,
            1,
            unique_agent_count(&single_skill_config),
            execution_summary.conflicts,
        );
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
    let mut discovered =
        match discover_remote_skills_via_temp_clone(&url_source.repo, &source_ref, &scope_subpath)
            .await
        {
            Ok(skills) => {
                clone_spinner.finish_success(ui);
                skills
            }
            Err(err) => {
                clone_spinner.finish_failure(ui, &err.to_string());
                return Err(err);
            }
        };
    if discovered.is_empty() {
        print_warning(ui, "No SKILL.md found; installing directory as-is.");
    }

    if req.list {
        print_discovered_skills(ui, &discovered);
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
    let resolved_targets = resolve_url_mode_install_targets(&req.target, ui)?;

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
        upsert_mode_a_skill(
            &mut config,
            &skill_id,
            &url_source.repo,
            &effective_subpath,
            &source_ref,
            &resolved_targets,
            requested_install_mode(req.copy),
        )?;
        selected_ids.push(skill_id);
    }

    validate_config(&config, &config_dir)?;
    let selected_config = select_config_skills(&config, &selected_ids);

    if req.dry_run {
        if let Some(skill_id) = selected_ids.first() {
            print_install_dry_run(
                ui,
                req.options.json,
                &selected_config,
                skill_id,
                &source_ref,
                &config_dir,
            )?;
            return Ok(());
        }
    }

    write_normalized_config(config_path, &config)?;

    let mut execution_summary = InstallExecutionSummary::default();
    for skill_id in &selected_ids {
        let single_skill_config = select_single_skill_config(&selected_config, skill_id)?;
        let sync_summary = sync_sources_async(&single_skill_config, &config_dir).await?;
        if !req.options.json {
            print_source_sync_summary(&sync_summary);
        }
        if let Some(err) = source_sync_failure_error(&sync_summary) {
            return Err(err);
        }
        let skill_summary =
            execute_install_plan(&single_skill_config, &config_dir, req.options.strict)?;
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
        print_install_result_summary(
            ui,
            selected_ids.len(),
            unique_agent_count(&selected_config),
            execution_summary.conflicts,
        );
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

    if req.list {
        print_discovered_skills(ui, &discovered);
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
    let resolved_targets = resolve_url_mode_install_targets(&req.target, ui)?;

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
        upsert_mode_a_skill(
            &mut config,
            &skill_id,
            &url_source.repo,
            &effective_subpath,
            &source_ref,
            &resolved_targets,
            requested_install_mode(req.copy),
        )?;
        selected_ids.push(skill_id);
    }

    validate_config(&config, &config_dir)?;
    let selected_config = select_config_skills(&config, &selected_ids);

    if req.dry_run {
        if let Some(skill_id) = selected_ids.first() {
            print_install_dry_run(
                ui,
                req.options.json,
                &selected_config,
                skill_id,
                &source_ref,
                &config_dir,
            )?;
            return Ok(());
        }
    }

    write_normalized_config(config_path, &config)?;

    let mut execution_summary = InstallExecutionSummary::default();
    for skill_id in &selected_ids {
        let single = select_single_skill_config(&selected_config, skill_id)?;
        let skill_summary = install_local_source_skill(&single, &config_dir, req.options.strict)?;
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
        print_install_result_summary(
            ui,
            selected_ids.len(),
            unique_agent_count(&selected_config),
            execution_summary.conflicts,
        );
    }

    Ok(())
}

async fn discover_remote_skills_via_temp_clone(
    repo_url: &str,
    reference: &str,
    scoped_subpath: &str,
) -> Result<Vec<DiscoveredSkill>, EdenError> {
    let repo_url = repo_url.to_string();
    let reference = reference.to_string();
    let scoped_subpath = scoped_subpath.to_string();
    tokio::task::spawn_blocking(move || {
        discover_remote_skills_via_temp_clone_blocking(&repo_url, &reference, &scoped_subpath)
    })
    .await
    .map_err(|err| EdenError::Runtime(format!("remote discovery worker failed: {err}")))?
}

fn discover_remote_skills_via_temp_clone_blocking(
    repo_url: &str,
    reference: &str,
    scoped_subpath: &str,
) -> Result<Vec<DiscoveredSkill>, EdenError> {
    let temp_checkout = create_discovery_temp_checkout()?;
    let repo_dir = temp_checkout.path.join("repo");
    clone_repo_for_discovery(repo_url, reference, &repo_dir)?;
    let discovery_root = normalize_lexical(&repo_dir.join(scoped_subpath));
    if !discovery_root.exists() {
        return Err(EdenError::Runtime(format!(
            "discovery path does not exist: {}",
            discovery_root.display()
        )));
    }
    discover_skills(&discovery_root)
}

struct TempDiscoveryCheckout {
    path: PathBuf,
}

impl Drop for TempDiscoveryCheckout {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn create_discovery_temp_checkout() -> Result<TempDiscoveryCheckout, EdenError> {
    for attempt in 0..10u32 {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| EdenError::Runtime(format!("system clock before unix epoch: {err}")))?
            .as_nanos();
        let candidate = std::env::temp_dir().join(format!(
            "eden-skills-discovery-{}-{unique}-{attempt}",
            std::process::id()
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(TempDiscoveryCheckout { path: candidate }),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(EdenError::Io(err)),
        }
    }
    Err(EdenError::Runtime(
        "failed to create temporary directory for remote discovery".to_string(),
    ))
}

fn clone_repo_for_discovery(
    repo_url: &str,
    reference: &str,
    repo_dir: &Path,
) -> Result<(), EdenError> {
    if let Some(parent) = repo_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    let branch_clone_result = run_git_command(
        Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--branch")
            .arg(reference)
            .arg(repo_url)
            .arg(repo_dir),
        &format!(
            "clone `{repo_url}` into `{}` with ref `{reference}`",
            repo_dir.display()
        ),
    );

    if let Err(branch_error) = branch_clone_result {
        let fallback_clone = run_git_command(
            Command::new("git").arg("clone").arg(repo_url).arg(repo_dir),
            &format!(
                "clone `{repo_url}` into `{}` without branch hint",
                repo_dir.display()
            ),
        );
        if let Err(fallback_error) = fallback_clone {
            return Err(EdenError::Runtime(format!(
                "branch clone attempt failed: {branch_error}; fallback clone attempt failed: {fallback_error}"
            )));
        }
        run_git_command(
            Command::new("git")
                .arg("-C")
                .arg(repo_dir)
                .arg("checkout")
                .arg(reference),
            &format!(
                "checkout ref `{reference}` in temporary discovery repo `{}`",
                repo_dir.display()
            ),
        )
        .map_err(EdenError::Runtime)?;
    }

    Ok(())
}

fn resolve_local_install_selection(
    discovered: &[DiscoveredSkill],
    all: bool,
    named: &[String],
    yes: bool,
    ui: &UiContext,
) -> Result<Vec<DiscoveredSkill>, EdenError> {
    if all || yes {
        return Ok(discovered.to_vec());
    }

    if !named.is_empty() {
        return select_named_skills(discovered, named);
    }

    if discovered.len() == 1 {
        return Ok(vec![discovered[0].clone()]);
    }

    if !ui.interactive_enabled() {
        return Ok(discovered.to_vec());
    }

    print_discovery_summary(ui, discovered);
    let install_all = prompt_install_all(discovered.len())?;
    if install_all {
        return Ok(discovered.to_vec());
    }

    let names = prompt_skill_names()?;
    if names.is_empty() {
        return Err(EdenError::InvalidArguments(
            "no skill names provided".to_string(),
        ));
    }
    select_named_skills(discovered, &names)
}

fn select_named_skills(
    discovered: &[DiscoveredSkill],
    names: &[String],
) -> Result<Vec<DiscoveredSkill>, EdenError> {
    let mut selected = Vec::new();
    let mut unknown = Vec::new();
    for name in names {
        if let Some(skill) = discovered.iter().find(|skill| skill.name == *name) {
            if !selected
                .iter()
                .any(|existing: &DiscoveredSkill| existing.name == skill.name)
            {
                selected.push(skill.clone());
            }
        } else {
            unknown.push(name.clone());
        }
    }

    if !unknown.is_empty() {
        let available = discovered
            .iter()
            .map(|skill| skill.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(EdenError::InvalidArguments(format!(
            "unknown skill name(s): {}; available: {}",
            unknown.join(", "),
            available
        )));
    }

    Ok(selected)
}

fn prompt_install_all(skill_count: usize) -> Result<bool, EdenError> {
    if let Ok(response) = std::env::var("EDEN_SKILLS_TEST_CONFIRM") {
        let normalized = response.trim().to_ascii_lowercase();
        return Ok(matches!(normalized.as_str(), "y" | "yes" | "true" | ""));
    }

    Confirm::new()
        .with_prompt(format!("Install all {skill_count} skills?"))
        .default(true)
        .interact()
        .map_err(|err| EdenError::Runtime(format!("interactive prompt failed: {err}")))
}

fn prompt_skill_names() -> Result<Vec<String>, EdenError> {
    if let Ok(raw) = std::env::var("EDEN_SKILLS_TEST_SKILL_INPUT") {
        return Ok(raw
            .split_whitespace()
            .map(ToString::to_string)
            .collect::<Vec<_>>());
    }

    let input: String = Input::new()
        .with_prompt("Enter skill names to install (space-separated)")
        .interact_text()
        .map_err(|err| EdenError::Runtime(format!("interactive prompt failed: {err}")))?;
    Ok(input
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>())
}

fn print_discovered_skills(ui: &UiContext, skills: &[DiscoveredSkill]) {
    const MAX_DISPLAY: usize = 8;

    println!(
        "{}  {} skills in repository:",
        ui.action_prefix("Found"),
        skills.len()
    );
    if skills.is_empty() {
        println!();
        println!("  (no SKILL.md discovered)");
        return;
    }

    println!();

    let display_skills = if skills.len() > MAX_DISPLAY {
        &skills[..MAX_DISPLAY]
    } else {
        skills
    };
    let mut table = ui.table(&["#", "Name", "Description"]);
    for (index, skill) in display_skills.iter().enumerate() {
        table.add_row(vec![
            (index + 1).to_string(),
            skill.name.clone(),
            skill.description.clone(),
        ]);
    }
    println!("{table}");

    if skills.len() > MAX_DISPLAY {
        println!(
            "  ... and {} more (use --list to see all)",
            skills.len() - MAX_DISPLAY
        );
    }
}

fn print_discovery_summary(ui: &UiContext, skills: &[DiscoveredSkill]) {
    const MAX_DISPLAY: usize = 8;
    if skills.len() > MAX_DISPLAY {
        println!(
            "{}  {} skills in repository (showing first {}):",
            ui.action_prefix("Found"),
            skills.len(),
            MAX_DISPLAY
        );
    } else {
        println!(
            "{}  {} skills in repository:",
            ui.action_prefix("Found"),
            skills.len()
        );
    }
    println!();

    let display_skills = if skills.len() > MAX_DISPLAY {
        &skills[..MAX_DISPLAY]
    } else {
        skills
    };
    let width = display_skills
        .iter()
        .map(|skill| skill.name.chars().count())
        .max()
        .unwrap_or(0);
    for (index, skill) in display_skills.iter().enumerate() {
        if skill.description.is_empty() {
            println!("    {}. {}", index + 1, skill.name);
        } else {
            println!(
                "    {}. {:<width$} — {}",
                index + 1,
                skill.name,
                skill.description
            );
        }
    }

    if skills.len() > MAX_DISPLAY {
        println!(
            "... and {} more (use --list to see all)",
            skills.len() - MAX_DISPLAY
        );
    }
}

fn join_scoped_subpath(scope_subpath: &str, discovered_subpath: &str) -> String {
    if scope_subpath == "." {
        return discovered_subpath.to_string();
    }
    if discovered_subpath == "." {
        return scope_subpath.to_string();
    }
    format!("{scope_subpath}/{discovered_subpath}")
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

fn print_install_dry_run(
    ui: &UiContext,
    json_mode: bool,
    single_skill_config: &Config,
    skill_id: &str,
    version_or_ref: &str,
    config_dir: &Path,
) -> Result<(), EdenError> {
    let resolved_skill = single_skill_config
        .skills
        .first()
        .ok_or_else(|| EdenError::Runtime("resolved install skill is missing".to_string()))?;

    if json_mode {
        let targets = resolved_skill
            .targets
            .iter()
            .map(|target| {
                let resolved_path = resolve_target_path(target, config_dir)
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|err| format!("ERROR: {err}"));
                serde_json::json!({
                    "agent": agent_kind_label(&target.agent),
                    "environment": target.environment,
                    "path": resolved_path,
                })
            })
            .collect::<Vec<_>>();
        let payload = serde_json::json!({
            "skill": skill_id,
            "version": version_or_ref,
            "dry_run": true,
            "resolved": {
                "repo": resolved_skill.source.repo.clone(),
                "ref": resolved_skill.source.r#ref.clone(),
                "subpath": resolved_skill.source.subpath.clone(),
            },
            "targets": targets,
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to encode install json: {err}")))?;
        println!("{encoded}");
    } else {
        let source_repo_display =
            abbreviate_home_path(&abbreviate_repo_url(&resolved_skill.source.repo));
        println!("{}  install preview", ui.action_prefix("Dry Run"));
        println!();
        println!("  Skill:   {skill_id}");
        println!("  Version: {version_or_ref}");
        println!(
            "  Source:  {} ({})",
            source_repo_display, resolved_skill.source.subpath
        );
        println!();

        let mut table = ui.table(&["Agent", "Path", "Mode"]);
        for target in &resolved_skill.targets {
            let resolved_path = resolve_target_path(target, config_dir)
                .map(|path| path.display().to_string())
                .unwrap_or_else(|err| format!("ERROR: {err}"));
            table.add_row(vec![
                agent_kind_label(&target.agent).to_string(),
                abbreviate_home_path(&resolved_path),
                resolved_skill.install.mode.as_str().to_string(),
            ]);
        }
        println!("{table}");
    }
    Ok(())
}

fn execute_install_plan(
    single_skill_config: &Config,
    config_dir: &Path,
    strict: bool,
) -> Result<InstallExecutionSummary, EdenError> {
    let plan = build_plan(single_skill_config, config_dir)?;
    let mut summary = InstallExecutionSummary::default();
    for item in &plan {
        match item.action {
            Action::Create | Action::Update => {
                apply_plan_item(item)?;
                summary.installed_targets.push(InstallTargetLine {
                    skill_id: item.skill_id.clone(),
                    target_path: item.target_path.clone(),
                    mode: item.install_mode.as_str().to_string(),
                });
            }
            Action::Conflict => summary.conflicts += 1,
            Action::Noop | Action::Remove => {}
        }
    }
    if strict && summary.conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "strict mode blocked install: {} conflict entries",
            summary.conflicts
        )));
    }
    Ok(summary)
}

fn install_local_source_skill(
    single_skill_config: &Config,
    config_dir: &Path,
    strict: bool,
) -> Result<InstallExecutionSummary, EdenError> {
    let skill = single_skill_config
        .skills
        .first()
        .ok_or_else(|| EdenError::Runtime("local install skill is missing".to_string()))?;
    let source_repo_root = resolve_path_string(&skill.source.repo, config_dir)?;
    let storage_root = resolve_path_string(&single_skill_config.storage_root, config_dir)?;
    let staged_repo_root = normalize_lexical(&storage_root.join(&skill.id));

    stage_local_source_into_storage(&source_repo_root, &staged_repo_root)?;
    let source_path = normalize_lexical(&staged_repo_root.join(&skill.source.subpath));
    if !source_path.exists() {
        return Err(EdenError::Runtime(format!(
            "source path missing for skill `{}`: {}",
            skill.id,
            source_path.display()
        )));
    }

    let mut summary = InstallExecutionSummary::default();
    for target in &skill.targets {
        let target_root = resolve_target_path(target, config_dir)?;
        let target_path = normalize_lexical(&target_root.join(&skill.id));
        match fs::symlink_metadata(&target_path) {
            Ok(metadata)
                if matches!(skill.install.mode, InstallMode::Symlink)
                    && !metadata.file_type().is_symlink() =>
            {
                summary.conflicts += 1;
                continue;
            }
            Ok(metadata)
                if matches!(skill.install.mode, InstallMode::Copy)
                    && metadata.file_type().is_symlink() =>
            {
                summary.conflicts += 1;
                continue;
            }
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(EdenError::Io(err)),
        }

        let item = eden_skills_core::plan::PlanItem {
            skill_id: skill.id.clone(),
            source_path: source_path.display().to_string(),
            target_path: target_path.display().to_string(),
            install_mode: skill.install.mode,
            action: Action::Update,
            reasons: vec![],
        };
        apply_plan_item(&item)?;
        summary.installed_targets.push(InstallTargetLine {
            skill_id: skill.id.clone(),
            target_path: target_path.display().to_string(),
            mode: skill.install.mode.as_str().to_string(),
        });
    }

    if strict && summary.conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "strict mode blocked install: {} conflict entries",
            summary.conflicts
        )));
    }

    Ok(summary)
}

fn stage_local_source_into_storage(
    source_repo_root: &Path,
    staged_repo_root: &Path,
) -> Result<(), EdenError> {
    if !source_repo_root.exists() {
        return Err(EdenError::Runtime(format!(
            "local source path does not exist: {}",
            source_repo_root.display()
        )));
    }

    ensure_parent_dir(staged_repo_root)?;
    if fs::symlink_metadata(staged_repo_root).is_ok() {
        remove_path(staged_repo_root)?;
    }
    copy_recursively(source_repo_root, staged_repo_root)?;
    Ok(())
}

fn print_install_success_json(skill_id: &str, version_or_ref: &str) -> Result<(), EdenError> {
    let payload = serde_json::json!({
        "skill": skill_id,
        "version": version_or_ref,
        "status": "installed",
    });
    let encoded = serde_json::to_string_pretty(&payload)
        .map_err(|err| EdenError::Runtime(format!("failed to encode install json: {err}")))?;
    println!("{encoded}");
    Ok(())
}

fn upsert_mode_b_skill(
    config: &mut Config,
    skill_name: &str,
    version_constraint: &str,
    registry: Option<&str>,
    target_specs: &[String],
    install_mode_override: Option<InstallMode>,
) -> Result<(), EdenError> {
    let install_mode = install_mode_override.unwrap_or_else(default_install_mode);
    let target_override = match target_specs {
        [] => None,
        [spec] => Some(parse_install_target_spec(spec)?),
        _ => return Err(EdenError::InvalidArguments(
            "registry-mode install accepts at most one --target (`local` or `docker:<container>`)"
                .to_string(),
        )),
    };
    if let Some(skill) = config
        .skills
        .iter_mut()
        .find(|skill| skill.id == skill_name)
    {
        skill.source = SourceConfig {
            repo: encode_registry_mode_repo(registry),
            subpath: ".".to_string(),
            r#ref: version_constraint.to_string(),
        };
        skill.install.mode = install_mode;
        skill.verify.enabled = true;
        skill.verify.checks = default_verify_checks_for_mode(install_mode);
        if let Some(target) = target_override {
            skill.targets = vec![target];
        }
        return Ok(());
    }

    let targets = if let Some(target) = target_override {
        vec![target]
    } else {
        vec![default_install_target()]
    };

    config.skills.push(SkillConfig {
        id: skill_name.to_string(),
        source: SourceConfig {
            repo: encode_registry_mode_repo(registry),
            subpath: ".".to_string(),
            r#ref: version_constraint.to_string(),
        },
        install: eden_skills_core::config::InstallConfig { mode: install_mode },
        targets,
        verify: eden_skills_core::config::VerifyConfig {
            enabled: true,
            checks: default_verify_checks_for_mode(install_mode),
        },
        safety: eden_skills_core::config::SafetyConfig {
            no_exec_metadata_only: false,
        },
    });
    Ok(())
}

fn upsert_mode_a_skill(
    config: &mut Config,
    skill_id: &str,
    repo: &str,
    subpath: &str,
    reference: &str,
    targets: &[TargetConfig],
    install_mode_override: Option<InstallMode>,
) -> Result<(), EdenError> {
    let install_mode = install_mode_override.unwrap_or_else(default_install_mode);
    let effective_targets = if targets.is_empty() {
        vec![default_install_target()]
    } else {
        targets.to_vec()
    };
    if let Some(skill) = config.skills.iter_mut().find(|skill| skill.id == skill_id) {
        skill.source = SourceConfig {
            repo: repo.to_string(),
            subpath: subpath.to_string(),
            r#ref: reference.to_string(),
        };
        skill.install.mode = install_mode;
        skill.verify.enabled = true;
        skill.verify.checks = default_verify_checks_for_mode(install_mode);
        skill.targets = effective_targets;
        return Ok(());
    }

    config.skills.push(SkillConfig {
        id: skill_id.to_string(),
        source: SourceConfig {
            repo: repo.to_string(),
            subpath: subpath.to_string(),
            r#ref: reference.to_string(),
        },
        install: eden_skills_core::config::InstallConfig { mode: install_mode },
        targets: effective_targets,
        verify: eden_skills_core::config::VerifyConfig {
            enabled: true,
            checks: default_verify_checks_for_mode(install_mode),
        },
        safety: eden_skills_core::config::SafetyConfig {
            no_exec_metadata_only: false,
        },
    });
    Ok(())
}

fn default_install_target() -> TargetConfig {
    TargetConfig {
        agent: AgentKind::ClaudeCode,
        expected_path: None,
        path: None,
        environment: "local".to_string(),
    }
}

fn default_install_mode() -> InstallMode {
    resolve_default_install_mode_decision().mode
}

fn requested_install_mode(copy: bool) -> Option<InstallMode> {
    if copy {
        Some(InstallMode::Copy)
    } else {
        None
    }
}

fn resolve_default_install_mode_decision() -> DefaultInstallModeDecision {
    if let Some(forced_symlink_supported) = forced_windows_symlink_support_for_tests() {
        return decide_default_install_mode(true, forced_symlink_supported);
    }

    #[cfg(windows)]
    {
        decide_default_install_mode(true, windows_supports_symlink_creation())
    }
    #[cfg(not(windows))]
    {
        decide_default_install_mode(false, true)
    }
}

fn decide_default_install_mode(
    is_windows: bool,
    symlink_supported: bool,
) -> DefaultInstallModeDecision {
    if is_windows && !symlink_supported {
        return DefaultInstallModeDecision {
            mode: InstallMode::Copy,
            warn_windows_hardcopy_fallback: true,
        };
    }
    DefaultInstallModeDecision {
        mode: InstallMode::Symlink,
        warn_windows_hardcopy_fallback: false,
    }
}

fn forced_windows_symlink_support_for_tests() -> Option<bool> {
    match std::env::var("EDEN_SKILLS_TEST_WINDOWS_SYMLINK_SUPPORTED")
        .ok()
        .as_deref()
    {
        Some("1") => Some(true),
        Some("0") => Some(false),
        _ => None,
    }
}

fn warn_windows_hardcopy_fallback_if_needed(ui: &UiContext, decision: DefaultInstallModeDecision) {
    if decision.warn_windows_hardcopy_fallback && !ui.json_mode() {
        print_warning(
            ui,
            "Windows symlink permission is unavailable; falling back to hardcopy mode; this may slow down installs.",
        );
    }
}

#[cfg(windows)]
fn windows_supports_symlink_creation() -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let probe_root = std::env::temp_dir().join(format!("eden-skills-symlink-probe-{nonce}"));
    let source_dir = probe_root.join("source");
    let link_dir = probe_root.join("link");

    let created = fs::create_dir_all(&source_dir)
        .and_then(|_| std::os::windows::fs::symlink_dir(&source_dir, &link_dir))
        .is_ok();

    let _ = fs::remove_dir_all(&probe_root);
    created
}

fn resolve_url_mode_install_targets(
    target_specs: &[String],
    ui: &UiContext,
) -> Result<Vec<TargetConfig>, EdenError> {
    if !target_specs.is_empty() {
        return parse_target_specs(target_specs);
    }

    let detected = detect_installed_agent_targets()?;
    if !detected.is_empty() {
        return Ok(detected);
    }

    print_warning(
        ui,
        "No installed agents detected; defaulting to claude-code (~/.claude/skills/)",
    );
    Ok(vec![default_install_target()])
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

fn print_install_result_lines(ui: &UiContext, installed_targets: &[InstallTargetLine]) {
    let mut install_prefix_emitted = false;
    for target in installed_targets {
        let prefix = if install_prefix_emitted {
            "          ".to_string()
        } else {
            install_prefix_emitted = true;
            format!("{}  ", ui.action_prefix("Install"))
        };
        println!(
            "{prefix}{} {} → {} ({})",
            ui.status_symbol(StatusSymbol::Success),
            target.skill_id,
            crate::ui::abbreviate_home_path(&target.target_path),
            target.mode
        );
    }
}

fn print_install_result_summary(
    ui: &UiContext,
    skill_count: usize,
    agent_count: usize,
    conflict_count: usize,
) {
    println!(
        "  {} {} skills installed to {} agents, {} conflicts",
        ui.status_symbol(StatusSymbol::Success),
        skill_count,
        agent_count,
        conflict_count
    );
}

fn parse_install_target_spec(spec: &str) -> Result<TargetConfig, EdenError> {
    if spec == "local" {
        return Ok(default_install_target());
    }
    if let Some(container_name) = spec.strip_prefix("docker:") {
        if container_name.trim().is_empty() {
            return Err(EdenError::InvalidArguments(
                "invalid --target `docker:`: container name is required".to_string(),
            ));
        }
        return Ok(TargetConfig {
            agent: AgentKind::ClaudeCode,
            expected_path: None,
            path: None,
            environment: spec.to_string(),
        });
    }
    Err(EdenError::InvalidArguments(format!(
        "invalid --target `{spec}` (expected `local` or `docker:<container>`)"
    )))
}
