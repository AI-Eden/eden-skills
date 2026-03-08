//! Install result display and summary formatting.

use owo_colors::OwoColorize;

use crate::ui::{StatusSymbol, UiContext};

use super::execute::InstallTargetLine;

pub(super) fn print_install_result_lines(ui: &UiContext, installed_targets: &[InstallTargetLine]) {
    let mut install_prefix_emitted = false;
    for (skill_id, targets) in group_install_targets(installed_targets) {
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
}

pub(super) fn print_install_result_summary(
    ui: &UiContext,
    summary: &super::execute::InstallExecutionSummary,
    agent_count: usize,
) {
    let installed = summary.installed_skill_count();
    let skipped = summary.skipped_skills;
    let conflicts = summary.conflicts;

    println!();
    let mut parts = vec![format!("{} installed", ui.styled_success_count(installed))];
    if skipped > 0 {
        parts.push(format!("{} skipped", ui.styled_skipped_count(skipped)));
    }
    if conflicts > 0 {
        parts.push(format!("{} conflicts", ui.styled_failed_count(conflicts)));
    }

    let symbol = if installed > 0 || skipped > 0 {
        ui.status_symbol(StatusSymbol::Success)
    } else {
        ui.status_symbol(StatusSymbol::Skipped)
    };
    println!(
        "  {symbol} {} ({} agents)",
        parts.join(", "),
        ui.styled_cyan(&agent_count.to_string())
    );
}

pub(super) fn print_docker_cp_hints(ui: &UiContext, containers: &[String]) {
    if containers.is_empty() {
        return;
    }
    println!();
    for container_name in containers {
        println!(
            "  {} Tip: add bind mounts for live sync. Run 'eden-skills docker mount-hint {}'.",
            ui.hint_prefix(),
            container_name
        );
    }
}

pub(super) fn style_skill_id(ui: &UiContext, skill_id: &str) -> String {
    ui.styled_skill_id(skill_id)
}

pub(super) fn style_mode_label(ui: &UiContext, mode: &str) -> String {
    let raw = format!("({mode})");
    ui.styled_secondary(&raw)
}

pub(super) fn style_tree_connector(ui: &UiContext, connector: &str) -> String {
    if ui.colors_enabled() {
        connector.dimmed().to_string()
    } else {
        connector.to_string()
    }
}

pub(super) fn group_install_targets(
    targets: &[InstallTargetLine],
) -> Vec<(String, Vec<&InstallTargetLine>)> {
    let mut groups: Vec<(String, Vec<&InstallTargetLine>)> = Vec::new();
    for target in targets {
        if let Some(group) = groups.last_mut().filter(|(id, _)| id == &target.skill_id) {
            group.1.push(target);
        } else {
            groups.push((target.skill_id.clone(), vec![target]));
        }
    }
    groups
}
