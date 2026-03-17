//! Configuration management: `init`, `list`, `add`, `set`, `config export`, and `config import`.
//!
//! These commands read or mutate `skills.toml` and its companion lock
//! file. None of them perform source sync, plan execution, or file
//! installation — they only affect the declarative config layer.

use std::collections::HashSet;
use std::fs;

use comfy_table::{ColumnConstraint, Width};
use eden_skills_core::config::SkillConfig;
use eden_skills_core::config::{
    config_dir_from_path, default_verify_checks_for_mode, validate_config,
};
use eden_skills_core::error::EdenError;
use eden_skills_core::lock::{lock_path_for_config, write_lock_file, LockFile};
use eden_skills_core::paths::{resolve_path_string, resolve_target_path};
use owo_colors::OwoColorize;

use super::common::{
    agent_kind_label, load_config_with_context, normalized_config_toml, parse_target_specs,
    print_warning, read_existing_registries, resolve_config_path, write_normalized_config,
};
use super::{AddRequest, CommandOptions, SetRequest};
use crate::ui::{abbreviate_home_path, abbreviate_repo_url, StatusSymbol, UiContext};

/// Create a new `skills.toml` and companion lock file.
///
/// Writes the default config template and an empty lock file.
/// Fails if the config already exists unless `force` is set.
///
/// # Errors
///
/// Returns [`EdenError::Conflict`] if the file exists without `force`,
/// or [`EdenError::Io`] on filesystem write failures.
pub fn init(config_path: &str, force: bool) -> Result<(), EdenError> {
    let ui = UiContext::from_env(false);
    let config_path = resolve_config_path(config_path)?;
    if config_path.exists() && !force {
        return Err(EdenError::Conflict(format!(
            "config already exists: {} (use --force to overwrite)",
            config_path.display()
        )));
    }

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&config_path, default_config_template())?;

    let lock_path = lock_path_for_config(&config_path);
    write_lock_file(&lock_path, &LockFile::empty())?;

    let display_path = ui.styled_path(&config_path.display().to_string());
    println!(
        "  {} Created config at {display_path}",
        ui.status_symbol(StatusSymbol::Success)
    );
    println!();
    println!("  Next steps:");
    print_init_next_step(
        &ui,
        "eden-skills install <owner/repo>",
        "Install skills from GitHub",
    );
    print_init_next_step(&ui, "eden-skills list", "Show configured skills");
    print_init_next_step(&ui, "eden-skills doctor", "Check installation health");
    Ok(())
}

fn print_init_next_step(ui: &UiContext, command: &str, description: &str) {
    let padded_command = format!("{command:<34}");
    if ui.colors_enabled() {
        println!("    {padded_command} {}", description.dimmed());
    } else {
        println!("    {padded_command} {description}");
    }
}

pub(crate) fn default_config_template() -> String {
    [
        "version = 1",
        "",
        "[storage]",
        "root = \"~/.eden-skills/skills\"",
        "",
    ]
    .join("\n")
}

/// List all configured skills and their targets.
///
/// Renders a table (`Skill | Mode | Source | Agents`) in human mode
/// or a JSON object with a `count` and `skills` array.
///
/// # Errors
///
/// Returns [`EdenError`] on config load or path resolution failures.
pub fn list(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, options.strict)?;
    let ui = UiContext::from_env(options.json);
    for warning in loaded.warnings {
        print_warning(&ui, &warning);
    }

    let config_dir = config_dir_from_path(config_path);
    let skills = &loaded.config.skills;

    if options.json {
        let payload = serde_json::json!({
            "count": skills.len(),
            "skills": skills.iter().map(|skill| {
                let targets = skill.targets.iter().map(|target| {
                    let resolved = resolve_target_path(target, &config_dir)
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|err| format!("ERROR: {err}"));

                    serde_json::json!({
                        "agent": agent_kind_label(&target.agent),
                        "path": resolved,
                    })
                }).collect::<Vec<_>>();

                serde_json::json!({
                    "id": skill.id,
                    "source": {
                        "repo": skill.source.repo,
                        "ref": skill.source.r#ref,
                        "subpath": skill.source.subpath,
                    },
                    "install": {
                        "mode": skill.install.mode.as_str(),
                    },
                    "verify": {
                        "enabled": skill.verify.enabled,
                        "checks": skill.verify.checks,
                    },
                    "targets": targets,
                })
            }).collect::<Vec<_>>(),
        });

        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to serialize list json: {err}")))?;
        println!("{encoded}");
        return Ok(());
    }

    println!(
        "{}  {} configured",
        ui.action_prefix("Skills"),
        skills.len()
    );
    if skills.is_empty() {
        return Ok(());
    }
    println!();

    let mut table = ui.table(&["Skill", "Mode", "Source", "Agents"]);
    if let Some(column) = table.column_mut(1) {
        column.set_constraint(ColumnConstraint::LowerBoundary(Width::Fixed(8)));
    }
    for skill in skills {
        let repo_display = abbreviate_home_path(&abbreviate_repo_url(&skill.source.repo));
        let source = format!("{repo_display} ({})", skill.source.subpath);
        table.add_row(vec![
            ui.styled_skill_id(&skill.id),
            ui.styled_secondary(skill.install.mode.as_str()),
            ui.styled_cyan(&source),
            render_skill_agents(&ui, skill, &config_dir),
        ]);
    }
    println!("{table}");

    Ok(())
}

fn render_skill_agents(
    ui: &UiContext,
    skill: &SkillConfig,
    config_dir: &std::path::Path,
) -> String {
    let mut seen = HashSet::new();
    let mut labels = Vec::new();
    for target in &skill.targets {
        let label = if target.agent.as_str() == "custom" {
            let resolved = resolve_target_path(target, config_dir)
                .map(|path| abbreviate_home_path(&path.display().to_string()))
                .unwrap_or_else(|_| {
                    target
                        .path
                        .as_ref()
                        .map_or_else(|| "unknown".to_string(), ToString::to_string)
                });
            format!("custom:{resolved}")
        } else {
            agent_kind_label(&target.agent).to_string()
        };
        if seen.insert(label.clone()) {
            labels.push(label);
        }
    }

    let visible_labels = labels.iter().take(5).cloned().collect::<Vec<_>>();
    let hidden_count = labels.len().saturating_sub(visible_labels.len());
    let mut rendered = visible_labels.join(", ");
    if hidden_count > 0 {
        if !rendered.is_empty() {
            rendered.push(' ');
        }
        rendered.push_str(&ui.styled_warning_text(&format!("+{hidden_count} more")));
    }
    if skill.safety.no_exec_metadata_only {
        if rendered.is_empty() {
            rendered = ui.styled_secondary("(metadata-only)");
        } else {
            rendered.push(' ');
            rendered.push_str(&ui.styled_secondary("(metadata-only)"));
        }
    }
    rendered
}

/// Add a new skill entry to `skills.toml`.
///
/// Validates that the ID is unique, parses target specs, writes the
/// updated config, and rebuilds the lock file.
///
/// # Errors
///
/// Returns [`EdenError::Conflict`] for duplicate IDs, [`EdenError::Validation`]
/// for invalid target specs, or [`EdenError::Io`] on write failures.
pub fn add(req: AddRequest) -> Result<(), EdenError> {
    let ui = UiContext::from_env(req.options.json);
    let config_path_buf = resolve_config_path(&req.config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, req.options.strict)?;
    for warning in loaded.warnings {
        print_warning(&ui, &warning);
    }

    let config_dir = config_dir_from_path(config_path);
    let mut config = loaded.config;

    if config.skills.iter().any(|s| s.id == req.id) {
        return Err(EdenError::InvalidArguments(format!(
            "skill id already exists: `{}`",
            req.id
        )));
    }

    let targets = parse_target_specs(&req.target_specs)?;
    let enabled = req.verify_enabled.unwrap_or(true);
    let checks = req
        .verify_checks
        .clone()
        .unwrap_or_else(|| default_verify_checks_for_mode(req.mode));

    let skill = SkillConfig {
        id: req.id.clone(),
        source: eden_skills_core::config::SourceConfig {
            repo: req.repo.clone(),
            subpath: req.subpath.clone(),
            r#ref: req.r#ref.clone(),
        },
        install: eden_skills_core::config::InstallConfig { mode: req.mode },
        targets,
        verify: eden_skills_core::config::VerifyConfig { enabled, checks },
        safety: eden_skills_core::config::SafetyConfig {
            no_exec_metadata_only: req.no_exec_metadata_only.unwrap_or(false),
        },
    };

    config.skills.push(skill);

    validate_config(&config, &config_dir)?;
    write_normalized_config(config_path, &config)?;

    if req.options.json {
        let payload = serde_json::json!({
            "action": "add",
            "config_path": config_path.display().to_string(),
            "skill_id": req.id,
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to serialize add json: {err}")))?;
        println!("{encoded}");
        return Ok(());
    }

    let styled_skill = style_quoted_skill_id(&ui, &req.id);
    let styled_path = ui.styled_path(&config_path.display().to_string());
    println!(
        "  {} Added {styled_skill} to {styled_path}",
        ui.status_symbol(StatusSymbol::Success)
    );
    Ok(())
}

/// Modify properties of an existing skill entry in `skills.toml`.
///
/// Applies partial updates (repo, ref, subpath, mode, targets, verify)
/// to the named skill. At least one mutation must be specified.
///
/// # Errors
///
/// Returns [`EdenError::InvalidArguments`] when no mutations are given,
/// [`EdenError::Conflict`] when the skill ID is not found, or
/// [`EdenError::Io`] on write failures.
pub fn set(req: SetRequest) -> Result<(), EdenError> {
    let has_any_mutation = req.repo.is_some()
        || req.r#ref.is_some()
        || req.subpath.is_some()
        || req.mode.is_some()
        || req.verify_enabled.is_some()
        || req.verify_checks.is_some()
        || req.target_specs.is_some()
        || req.no_exec_metadata_only.is_some();
    if !has_any_mutation {
        return Err(EdenError::InvalidArguments(
            "set requires at least one mutation flag".to_string(),
        ));
    }

    let ui = UiContext::from_env(req.options.json);
    let config_path_buf = resolve_config_path(&req.config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, req.options.strict)?;
    for warning in loaded.warnings {
        print_warning(&ui, &warning);
    }

    let config_dir = config_dir_from_path(config_path);
    let mut config = loaded.config;

    let Some(skill) = config.skills.iter_mut().find(|s| s.id == req.skill_id) else {
        return Err(EdenError::InvalidArguments(format!(
            "unknown skill id: `{}`",
            req.skill_id
        )));
    };

    if let Some(repo) = req.repo {
        skill.source.repo = repo;
    }
    if let Some(r#ref) = req.r#ref {
        skill.source.r#ref = r#ref;
    }
    if let Some(subpath) = req.subpath {
        skill.source.subpath = subpath;
    }
    if let Some(mode) = req.mode {
        skill.install.mode = mode;
    }
    if let Some(enabled) = req.verify_enabled {
        skill.verify.enabled = enabled;
    }
    if let Some(checks) = req.verify_checks {
        skill.verify.checks = checks;
    }
    if let Some(target_specs) = req.target_specs {
        skill.targets = parse_target_specs(&target_specs)?;
    }
    if let Some(flag) = req.no_exec_metadata_only {
        skill.safety.no_exec_metadata_only = flag;
    }

    validate_config(&config, &config_dir)?;
    write_normalized_config(config_path, &config)?;

    if req.options.json {
        let payload = serde_json::json!({
            "action": "set",
            "config_path": config_path.display().to_string(),
            "skill_id": req.skill_id,
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to serialize set json: {err}")))?;
        println!("{encoded}");
        return Ok(());
    }

    let styled_skill = style_quoted_skill_id(&ui, &req.skill_id);
    let styled_path = ui.styled_path(&config_path.display().to_string());
    println!(
        "  {} Updated {styled_skill} in {styled_path}",
        ui.status_symbol(StatusSymbol::Success)
    );
    Ok(())
}

/// Export the current configuration to stdout as TOML or JSON.
///
/// # Errors
///
/// Returns [`EdenError`] on config load or serialization failures.
pub fn config_export(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let ui = UiContext::from_env(options.json);
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, options.strict)?;
    for warning in loaded.warnings {
        print_warning(&ui, &warning);
    }

    let registries = read_existing_registries(config_path)?;
    let toml = normalized_config_toml(&loaded.config, registries.as_ref());

    if options.json {
        let payload = serde_json::json!({
            "format": "toml",
            "toml": toml,
        });
        let encoded = serde_json::to_string_pretty(&payload).map_err(|err| {
            EdenError::Runtime(format!("failed to serialize config export json: {err}"))
        })?;
        println!("{encoded}");
        return Ok(());
    }

    print!("{toml}");
    Ok(())
}

/// Import configuration from another file, merging skills into the
/// current config while preserving existing entries.
///
/// In dry-run mode, previews the merge without writing changes.
///
/// # Errors
///
/// Returns [`EdenError`] on source or target config load failures,
/// validation errors in the merged config, or filesystem write errors.
pub fn config_import(
    from_path: &str,
    config_path: &str,
    dry_run: bool,
    options: CommandOptions,
) -> Result<(), EdenError> {
    let ui = UiContext::from_env(options.json);
    let cwd = std::env::current_dir().map_err(EdenError::Io)?;
    let from_path = resolve_path_string(from_path, &cwd)?;
    let loaded = load_config_with_context(&from_path, options.strict)?;
    for warning in loaded.warnings {
        print_warning(&ui, &warning);
    }

    let registries = read_existing_registries(&from_path)?;
    let toml = normalized_config_toml(&loaded.config, registries.as_ref());

    if dry_run {
        print!("{toml}");
        return Ok(());
    }

    let dest_path = resolve_config_path(config_path)?;
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&dest_path, toml)?;
    let styled_path = ui.styled_path(&dest_path.display().to_string());
    println!(
        "  {} Imported config to {styled_path}",
        ui.status_symbol(StatusSymbol::Success)
    );
    Ok(())
}

fn style_quoted_skill_id(ui: &UiContext, skill_id: &str) -> String {
    if ui.colors_enabled() {
        format!("'{}'", skill_id.bold())
    } else {
        format!("'{skill_id}'")
    }
}
