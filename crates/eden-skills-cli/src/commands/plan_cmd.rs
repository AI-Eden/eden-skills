//! Read-only plan preview via the `plan` command.
//!
//! Computes the lock diff and builds an action plan without performing
//! any side effects. Renders as colored text for small plans or as a
//! table when the action count exceeds a threshold.

use comfy_table::{ColumnConstraint, Width};
use eden_skills_core::config::{config_dir_from_path, InstallMode};
use eden_skills_core::error::EdenError;
use eden_skills_core::lock::{
    compute_lock_diff, lock_path_for_config, read_lock_file, LockSkillEntry,
};
use eden_skills_core::plan::{build_plan, Action, PlanItem};
use owo_colors::OwoColorize;

use super::common::{load_config_with_context, print_warning, resolve_config_path};
use super::CommandOptions;
use crate::ui::{StatusSymbol, UiContext};

/// Preview planned reconciliation actions without side effects.
///
/// Computes the lock diff and builds an action plan. Integrates
/// lock-based remove items for orphaned entries. Renders as colored
/// text (≤ 5 actions) or table (> 5 actions) in human mode, or JSON.
///
/// # Errors
///
/// Returns [`EdenError`] on config load, lock read, or plan build failures.
pub fn plan(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, options.strict)?;
    let ui = UiContext::from_env(options.json);
    for warning in &loaded.warnings {
        print_warning(&ui, warning);
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
        print_plan_text(&ui, &plan);
    }
    Ok(())
}

pub(crate) fn print_plan_text(ui: &UiContext, items: &[PlanItem]) {
    const TABLE_THRESHOLD: usize = 5;
    let has_pending_action = items
        .iter()
        .any(|item| !matches!(item.action, Action::Noop));
    if items.is_empty() || !has_pending_action {
        println!(
            "{}  {} 0 actions (up to date)",
            ui.action_prefix("Plan"),
            ui.status_symbol(StatusSymbol::Success),
        );
        return;
    }
    println!("{}  {} actions", ui.action_prefix("Plan"), items.len());
    println!();

    if items.len() > TABLE_THRESHOLD {
        print_plan_table(ui, items);
        return;
    }

    for item in items {
        let mode_label = style_mode_label(ui, item.install_mode.as_str());
        println!(
            "  {}  {} {} {} {}",
            style_plan_action_label(ui, item.action),
            ui.styled_skill_id(&item.skill_id),
            style_arrow(ui),
            ui.styled_path(&item.target_path),
            mode_label
        );
        for reason in &item.reasons {
            println!("           reason: {reason}");
        }
    }
}

fn print_plan_table(ui: &UiContext, items: &[PlanItem]) {
    let mut table = ui.table(&["Action", "Skill", "Target", "Mode"]);
    if let Some(column) = table.column_mut(0) {
        column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(10)));
    }
    if let Some(column) = table.column_mut(3) {
        column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(8)));
    }
    for item in items {
        table.add_row(vec![
            plan_action_cell(item.action),
            ui.styled_skill_id(&item.skill_id),
            ui.styled_path(&item.target_path),
            ui.styled_secondary(item.install_mode.as_str()),
        ]);
    }
    println!("{table}");

    let conflicts = items
        .iter()
        .filter(|item| matches!(item.action, Action::Conflict) && !item.reasons.is_empty())
        .collect::<Vec<_>>();
    if conflicts.is_empty() {
        return;
    }

    println!();
    println!("  Conflicts:");
    for item in conflicts {
        println!(
            "    {} {} {}",
            ui.styled_skill_id(&item.skill_id),
            style_arrow(ui),
            ui.styled_path(&item.target_path)
        );
        for reason in &item.reasons {
            println!("      reason: {reason}");
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

fn style_plan_action_label(ui: &UiContext, action: Action) -> String {
    let padded = format!("{:>8}", action_label(action));
    if !ui.colors_enabled() {
        return padded;
    }
    match action {
        Action::Create => padded.green().to_string(),
        Action::Update => padded.cyan().to_string(),
        Action::Noop => padded.dimmed().to_string(),
        Action::Conflict => padded.yellow().to_string(),
        Action::Remove => padded.red().to_string(),
    }
}

fn style_mode_label(ui: &UiContext, mode: &str) -> String {
    let raw = format!("({mode})");
    ui.styled_secondary(&raw)
}

fn style_arrow(ui: &UiContext) -> String {
    ui.hint_prefix()
}

fn plan_action_cell(action: Action) -> String {
    action_label(action).to_string()
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
