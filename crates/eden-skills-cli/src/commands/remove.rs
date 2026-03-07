//! Skill removal: `remove` and batch `remove_many_async`.
//!
//! Validates requested IDs against the config, prompts for interactive
//! confirmation (when applicable), uninstalls targets via the adapter,
//! removes config entries, and updates the lock file atomically.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::ui::{
    prompt_skill_multi_select, SkillSelectItem, SkillSelectOutcome, StatusSymbol, UiContext,
};
use dialoguer::Confirm;
use eden_skills_core::adapter::create_adapter;
use eden_skills_core::config::{config_dir_from_path, validate_config, Config, SkillConfig};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::{
    known_default_agent_paths, normalize_lexical, resolve_path_string, resolve_target_path,
};

use super::common::{
    block_on_command_future, ensure_docker_available_for_targets, format_quoted_ids,
    load_config_with_context, print_warning, remove_path, resolve_config_path, unique_ids,
    with_hint, write_lock_for_config, write_normalized_config,
};
use super::CommandOptions;

/// Remove a single skill, skipping interactive confirmation.
///
/// # Errors
///
/// Returns [`EdenError`] if the skill ID is not found or uninstall fails.
pub fn remove(config_path: &str, skill_id: &str, options: CommandOptions) -> Result<(), EdenError> {
    let skill_ids = vec![skill_id.to_string()];
    block_on_command_future(remove_many_async(config_path, &skill_ids, true, options))
}

/// Async variant of [`remove`] for a single skill.
///
/// # Errors
///
/// Returns [`EdenError`] if the skill ID is not found or uninstall fails.
pub async fn remove_async(
    config_path: &str,
    skill_id: &str,
    options: CommandOptions,
) -> Result<(), EdenError> {
    let skill_ids = vec![skill_id.to_string()];
    remove_many_async(config_path, &skill_ids, true, options).await
}

/// Remove one or more skills by ID with optional interactive confirmation.
///
/// Validates all IDs against the config atomically, prompts the user
/// when `skip_confirmation` is false, uninstalls targets via adapters,
/// removes config entries, and updates the lock file.
///
/// # Errors
///
/// Returns [`EdenError::Conflict`] for unknown skill IDs,
/// [`EdenError::Runtime`] on adapter failures, or [`EdenError::Io`]
/// on config/lock write errors.
pub async fn remove_many_async(
    config_path: &str,
    skill_ids: &[String],
    skip_confirmation: bool,
    options: CommandOptions,
) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, options.strict)?;
    let ui = UiContext::from_env(options.json);
    for warning in loaded.warnings {
        print_warning(&ui, &warning);
    }

    let config_dir = config_dir_from_path(config_path);
    let mut config = loaded.config;
    let (removal_ids, prompted) = match resolve_remove_ids(&config, skill_ids, &ui)? {
        RemoveSelection::Cancelled => {
            print_remove_cancelled(&ui);
            return Ok(());
        }
        RemoveSelection::Interrupted => {
            print_remove_interrupted(&ui);
            return Ok(());
        }
        RemoveSelection::SkillIds { ids, prompted } => (ids, prompted),
    };
    if removal_ids.is_empty() {
        return Ok(());
    }
    validate_remove_ids(&config, &removal_ids, skill_ids.len() == 1)?;

    match confirm_remove_execution(&removal_ids, prompted, skip_confirmation, &ui)? {
        PromptOutcome::Interrupted => {
            print_remove_interrupted(&ui);
            return Ok(());
        }
        PromptOutcome::Cancelled | PromptOutcome::Value(false) => {
            print_remove_cancelled(&ui);
            return Ok(());
        }
        PromptOutcome::Value(true) => {}
    }

    let mut removed = Vec::with_capacity(removal_ids.len());
    for skill_id in &removal_ids {
        let idx = config
            .skills
            .iter()
            .position(|s| s.id == *skill_id)
            .ok_or_else(|| {
                EdenError::Runtime(format!(
                    "validated skill id disappeared during removal: {skill_id}"
                ))
            })?;
        let removed_skill = config.skills[idx].clone();
        ensure_docker_available_for_targets(
            removed_skill
                .targets
                .iter()
                .map(|target| target.environment.as_str()),
        )?;
        uninstall_skill_targets(&removed_skill, &config_dir, &config.storage_root).await?;
        config.skills.remove(idx);
        removed.push(skill_id.clone());
    }

    validate_config(&config, &config_dir)?;
    write_normalized_config(config_path, &config)?;
    write_lock_for_config(config_path, &config, &config_dir)?;

    if options.json {
        let payload = serde_json::json!({
            "action": "remove",
            "config_path": config_path.display().to_string(),
            "removed": removed,
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to serialize remove json: {err}")))?;
        println!("{encoded}");
        return Ok(());
    }

    print_remove_summary(&ui, &removed);
    Ok(())
}

enum RemoveSelection {
    SkillIds { ids: Vec<String>, prompted: bool },
    Cancelled,
    Interrupted,
}

enum PromptOutcome<T> {
    Value(T),
    Cancelled,
    Interrupted,
}

fn resolve_remove_ids(
    config: &Config,
    skill_ids: &[String],
    ui: &UiContext,
) -> Result<RemoveSelection, EdenError> {
    if !skill_ids.is_empty() {
        return Ok(RemoveSelection::SkillIds {
            ids: unique_ids(skill_ids),
            prompted: false,
        });
    }

    if config.skills.is_empty() {
        println!("{}   0 configured", ui.action_prefix("Skills"));
        println!();
        println!("  Nothing to remove.");
        return Ok(RemoveSelection::SkillIds {
            ids: Vec::new(),
            prompted: false,
        });
    }

    if !ui.interactive_enabled() {
        return Err(EdenError::InvalidArguments(with_hint(
            "no skill IDs specified",
            "Usage: eden-skills remove <SKILL_ID>...",
        )));
    }

    let selection = match prompt_remove_selection(config, ui)? {
        PromptOutcome::Cancelled => return Ok(RemoveSelection::Cancelled),
        PromptOutcome::Interrupted => return Ok(RemoveSelection::Interrupted),
        PromptOutcome::Value(selection) => selection,
    };
    if selection.is_empty() {
        return Err(EdenError::InvalidArguments(with_hint(
            "no skill IDs specified",
            "Usage: eden-skills remove <SKILL_ID>...",
        )));
    }
    let ids = selection
        .into_iter()
        .map(|index| config.skills[index].id.clone())
        .collect();
    Ok(RemoveSelection::SkillIds {
        ids,
        prompted: true,
    })
}

fn validate_remove_ids(
    config: &Config,
    removal_ids: &[String],
    keep_single_unknown_message: bool,
) -> Result<(), EdenError> {
    let known = config
        .skills
        .iter()
        .map(|skill| skill.id.as_str())
        .collect::<HashSet<_>>();
    let mut unknown = Vec::new();
    for id in removal_ids {
        if !known.contains(id.as_str()) {
            unknown.push(id.clone());
        }
    }
    if unknown.is_empty() {
        return Ok(());
    }

    let available = config
        .skills
        .iter()
        .map(|skill| skill.id.clone())
        .collect::<Vec<_>>();
    let hint = if available.is_empty() {
        "Available skills: (none configured)".to_string()
    } else {
        format!("Available skills: {}", available.join(", "))
    };

    let message = if keep_single_unknown_message && unknown.len() == 1 {
        format!("skill '{}' not found in config", unknown[0])
    } else {
        format!("unknown skill(s): {}", format_quoted_ids(&unknown))
    };
    Err(EdenError::InvalidArguments(with_hint(message, hint)))
}

fn confirm_remove_execution(
    removal_ids: &[String],
    prompted: bool,
    skip_confirmation: bool,
    ui: &UiContext,
) -> Result<PromptOutcome<bool>, EdenError> {
    if skip_confirmation || !ui.interactive_enabled() {
        return Ok(PromptOutcome::Value(true));
    }

    if let Ok(response) = std::env::var("EDEN_SKILLS_TEST_CONFIRM") {
        let normalized = response.trim().to_ascii_lowercase();
        if normalized == "interrupt" {
            return Ok(PromptOutcome::Interrupted);
        }
        return Ok(PromptOutcome::Value(matches!(
            normalized.as_str(),
            "y" | "yes" | "true"
        )));
    }

    let _interrupt_guard = crate::signal::PromptInterruptGuard::new();
    match Confirm::new()
        .with_prompt(remove_confirmation_prompt(removal_ids, prompted))
        .default(false)
        .interact()
    {
        Ok(value) => {
            if crate::signal::take_prompt_interrupt() {
                Ok(PromptOutcome::Interrupted)
            } else {
                Ok(PromptOutcome::Value(value))
            }
        }
        Err(dialoguer::Error::IO(err)) if err.kind() == std::io::ErrorKind::Interrupted => {
            if crate::signal::take_prompt_interrupt() {
                Ok(PromptOutcome::Interrupted)
            } else {
                Ok(PromptOutcome::Cancelled)
            }
        }
        Err(err) => {
            if crate::signal::take_prompt_interrupt() {
                Ok(PromptOutcome::Interrupted)
            } else {
                Err(EdenError::Runtime(format!(
                    "interactive prompt failed: {err}"
                )))
            }
        }
    }
}

fn print_remove_summary(ui: &UiContext, removed: &[String]) {
    if removed.is_empty() {
        return;
    }

    let success = ui.status_symbol(StatusSymbol::Success);
    println!(
        "{}  {} {}",
        ui.action_prefix("Remove"),
        success,
        ui.styled_skill_id(&removed[0])
    );
    for skill_id in removed.iter().skip(1) {
        println!("          {} {}", success, ui.styled_skill_id(skill_id));
    }
    println!();
    let noun = if removed.len() == 1 {
        "skill"
    } else {
        "skills"
    };
    println!("  {} {} {} removed", success, removed.len(), noun);
}

fn remove_confirmation_prompt(removal_ids: &[String], prompted: bool) -> String {
    if prompted {
        let noun = if removal_ids.len() == 1 {
            "skill"
        } else {
            "skills"
        };
        return format!("Remove {} {noun}?", removal_ids.len());
    }

    format!("Remove {}?", removal_ids.join(", "))
}

fn print_remove_cancelled(ui: &UiContext) {
    if !ui.json_mode() {
        println!(
            "  {} Remove cancelled",
            ui.status_symbol(StatusSymbol::Skipped)
        );
    }
}

fn print_remove_interrupted(ui: &UiContext) {
    if !ui.json_mode() {
        println!();
        println!("{}", ui.signal_cancelled_line("Remove"));
    }
}

fn prompt_remove_selection(
    config: &Config,
    ui: &UiContext,
) -> Result<PromptOutcome<Vec<usize>>, EdenError> {
    let items = config
        .skills
        .iter()
        .map(|skill| SkillSelectItem {
            name: skill.id.as_str(),
            description: "",
        })
        .collect::<Vec<_>>();
    match prompt_skill_multi_select(
        ui,
        "Select skills to remove",
        &items,
        "EDEN_SKILLS_TEST_REMOVE_INPUT",
        None,
    )? {
        SkillSelectOutcome::Selected(indices) => Ok(PromptOutcome::Value(indices)),
        SkillSelectOutcome::Interrupted => Ok(PromptOutcome::Interrupted),
        SkillSelectOutcome::Cancelled => Ok(PromptOutcome::Cancelled),
    }
}

async fn uninstall_skill_targets(
    skill: &SkillConfig,
    config_dir: &Path,
    storage_root: &str,
) -> Result<(), EdenError> {
    for target in &skill.targets {
        let target_root = resolve_target_path(target, config_dir)?;
        let installed_target = normalize_lexical(&target_root.join(&skill.id));
        let adapter = create_adapter(&target.environment).map_err(EdenError::from)?;
        adapter
            .uninstall(&installed_target)
            .await
            .map_err(EdenError::from)?;
    }
    remove_from_known_local_agent_targets(&skill.id, config_dir)?;
    remove_from_storage_root(skill, storage_root, config_dir)?;
    Ok(())
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
        let candidate = normalize_lexical(&resolved_root.join(skill_id));
        if fs::symlink_metadata(&candidate).is_ok() {
            remove_path(&candidate)?;
        }
    }
    Ok(())
}

fn remove_from_storage_root(
    skill: &SkillConfig,
    storage_root: &str,
    config_dir: &Path,
) -> Result<(), EdenError> {
    let resolved_storage_root = resolve_path_string(storage_root, config_dir)?;
    let canonical_skill_dir = normalize_lexical(&resolved_storage_root.join(&skill.id));
    if fs::symlink_metadata(&canonical_skill_dir).is_ok() {
        remove_path(&canonical_skill_dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::remove_confirmation_prompt;

    #[test]
    fn remove_confirmation_prompt_counts_prompted_multi_select_items() {
        let prompt = remove_confirmation_prompt(&["a".to_string(), "b".to_string()], true);
        assert_eq!(prompt, "Remove 2 skills?");
    }

    #[test]
    fn remove_confirmation_prompt_lists_explicit_skill_ids() {
        let prompt = remove_confirmation_prompt(&["a".to_string(), "b".to_string()], false);
        assert_eq!(prompt, "Remove a, b?");
    }
}
