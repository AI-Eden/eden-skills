use eden_skills_core::config::{config_dir_from_path, InstallMode};
use eden_skills_core::error::EdenError;
use eden_skills_core::lock::{
    compute_lock_diff, lock_path_for_config, read_lock_file, LockSkillEntry,
};
use eden_skills_core::plan::{build_plan, Action, PlanItem};

use super::common::{load_config_with_context, resolve_config_path};
use super::CommandOptions;

pub fn plan(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, options.strict)?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }

    let config_dir = config_dir_from_path(config_path);
    let lock_path = lock_path_for_config(config_path);
    let lock = read_lock_file(&lock_path)?;
    let diff = compute_lock_diff(&loaded.config, &lock, &config_dir)?;

    let mut plan = build_plan(&loaded.config, &config_dir)?;
    plan.extend(build_remove_plan_items(&diff.removed));

    if options.json {
        print_plan_json(&plan)?;
    } else {
        print_plan_text(&plan);
    }
    Ok(())
}

pub(crate) fn print_plan_text(items: &[PlanItem]) {
    if items.is_empty() {
        println!("plan: 0 actions");
        return;
    }

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

pub(crate) fn print_plan_json(items: &[PlanItem]) -> Result<(), EdenError> {
    let payload = serde_json::to_string_pretty(items)
        .map_err(|err| EdenError::Runtime(format!("failed to serialize plan as json: {err}")))?;
    println!("{payload}");
    Ok(())
}

pub(crate) fn action_label(action: Action) -> &'static str {
    match action {
        Action::Create => "create",
        Action::Update => "update",
        Action::Noop => "noop",
        Action::Conflict => "conflict",
        Action::Remove => "remove",
    }
}

pub(crate) fn build_remove_plan_items(removed: &[LockSkillEntry]) -> Vec<PlanItem> {
    let mut items = Vec::new();
    for entry in removed {
        for target in &entry.targets {
            items.push(PlanItem {
                skill_id: entry.id.clone(),
                source_path: String::new(),
                target_path: target.path.clone(),
                install_mode: if entry.install_mode == "copy" {
                    InstallMode::Copy
                } else {
                    InstallMode::Symlink
                },
                action: Action::Remove,
                reasons: vec!["skill removed from configuration".to_string()],
            });
        }
    }
    items
}
