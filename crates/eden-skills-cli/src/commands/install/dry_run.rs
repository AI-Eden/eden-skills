//! Dry-run preview tables for install operations.

use std::collections::HashSet;
use std::path::Path;

use comfy_table::ColumnConstraint;
use comfy_table::Width;
use eden_skills_core::config::Config;
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::resolve_target_path;
use owo_colors::OwoColorize;

use crate::ui::{abbreviate_home_path, abbreviate_repo_url, UiContext};

use crate::commands::common::agent_kind_label;

pub(super) const DRY_RUN_SKILL_PREVIEW_LIMIT: usize = 8;
pub(super) const DRY_RUN_TABLE_INDENT: usize = 4;

#[derive(Debug, Clone)]
pub(super) struct DryRunSkillPreviewRow {
    pub(super) skill_id: String,
    pub(super) version: String,
    pub(super) source: String,
}

#[derive(Debug, Clone)]
pub(super) struct DryRunTargetPreviewRow {
    pub(super) agent: String,
    pub(super) path: String,
    pub(super) mode: String,
}

#[derive(Debug, Clone)]
pub(super) struct DryRunPreviewData {
    pub(super) skills: Vec<DryRunSkillPreviewRow>,
    pub(super) targets: Vec<DryRunTargetPreviewRow>,
}

pub(super) fn print_install_dry_run(
    ui: &UiContext,
    json_mode: bool,
    resolved_config: &Config,
    skill_ids: &[String],
    config_dir: &Path,
    show_all_skills: bool,
) -> Result<(), EdenError> {
    let preview_data = build_dry_run_preview_data(resolved_config, skill_ids, config_dir)?;

    if json_mode {
        return print_install_dry_run_json(resolved_config, skill_ids, config_dir);
    }

    println!("{}  install preview", ui.action_prefix("Dry Run"));
    println!();

    let display_count = if show_all_skills {
        preview_data.skills.len()
    } else {
        preview_data.skills.len().min(DRY_RUN_SKILL_PREVIEW_LIMIT)
    };
    let displayed_skills = &preview_data.skills[..display_count];

    let mut skill_table = ui.table(&["#", "Skill", "Version", "Source"]);
    if let Some(column) = skill_table.column_mut(0) {
        column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(4)));
    }
    for (index, row) in displayed_skills.iter().enumerate() {
        skill_table.add_row(vec![
            (index + 1).to_string(),
            ui.styled_skill_id(&row.skill_id),
            ui.styled_version(&row.version),
            ui.styled_cyan(&row.source),
        ]);
    }
    print_titled_table(ui, "Skill / Version / Source", &skill_table);
    if !show_all_skills && preview_data.skills.len() > DRY_RUN_SKILL_PREVIEW_LIMIT {
        let footer = format!(
            "    ... and {} more (use --dry-run --list to show all)",
            preview_data.skills.len() - DRY_RUN_SKILL_PREVIEW_LIMIT
        );
        if ui.colors_enabled() {
            println!("{}", footer.dimmed());
        } else {
            println!("{footer}");
        }
    }

    println!();
    let mut target_table = ui.table(&["Agent", "Path", "Mode"]);
    if let Some(column) = target_table.column_mut(2) {
        column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(8)));
    }
    for row in &preview_data.targets {
        target_table.add_row(vec![
            row.agent.clone(),
            ui.styled_path(&row.path),
            ui.styled_secondary(&row.mode),
        ]);
    }
    print_titled_table(ui, "Install Targets", &target_table);

    Ok(())
}

fn build_dry_run_preview_data(
    resolved_config: &Config,
    skill_ids: &[String],
    config_dir: &Path,
) -> Result<DryRunPreviewData, EdenError> {
    let mut skill_rows = Vec::new();
    let mut target_rows = Vec::new();
    let mut seen_target_rows = HashSet::new();

    for skill_id in skill_ids {
        let skill = resolved_config
            .skills
            .iter()
            .find(|candidate| &candidate.id == skill_id)
            .ok_or_else(|| {
                EdenError::Runtime(format!("resolved install skill is missing: `{skill_id}`"))
            })?;

        let source_repo_display = abbreviate_home_path(&abbreviate_repo_url(&skill.source.repo));
        skill_rows.push(DryRunSkillPreviewRow {
            skill_id: skill.id.clone(),
            version: skill.source.r#ref.clone(),
            source: format!("{source_repo_display} ({})", skill.source.subpath),
        });

        for target in &skill.targets {
            let resolved_path = resolve_target_path(target, config_dir)
                .map(|path| path.display().to_string())
                .unwrap_or_else(|err| format!("ERROR: {err}"));
            let row = DryRunTargetPreviewRow {
                agent: agent_kind_label(&target.agent).to_string(),
                path: abbreviate_home_path(&resolved_path),
                mode: skill.install.mode.as_str().to_string(),
            };
            let key = format!("{}|{}|{}", row.agent, row.path, row.mode);
            if seen_target_rows.insert(key) {
                target_rows.push(row);
            }
        }
    }

    Ok(DryRunPreviewData {
        skills: skill_rows,
        targets: target_rows,
    })
}

fn print_install_dry_run_json(
    resolved_config: &Config,
    skill_ids: &[String],
    config_dir: &Path,
) -> Result<(), EdenError> {
    let mut skill_payloads = Vec::new();
    for skill_id in skill_ids {
        let resolved_skill = resolved_config
            .skills
            .iter()
            .find(|candidate| &candidate.id == skill_id)
            .ok_or_else(|| {
                EdenError::Runtime(format!("resolved install skill is missing: `{skill_id}`"))
            })?;
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
        skill_payloads.push(serde_json::json!({
            "skill": skill_id,
            "version": resolved_skill.source.r#ref,
            "resolved": {
                "repo": resolved_skill.source.repo,
                "ref": resolved_skill.source.r#ref,
                "subpath": resolved_skill.source.subpath,
            },
            "targets": targets,
        }));
    }

    let payload = if skill_payloads.len() == 1 {
        let mut single = skill_payloads
            .into_iter()
            .next()
            .ok_or_else(|| EdenError::Runtime("resolved install skill is missing".to_string()))?;
        if let Some(map) = single.as_object_mut() {
            map.insert("dry_run".to_string(), serde_json::json!(true));
        }
        single
    } else {
        serde_json::json!({
            "dry_run": true,
            "skills": skill_payloads,
        })
    };

    let encoded = serde_json::to_string_pretty(&payload)
        .map_err(|err| EdenError::Runtime(format!("failed to encode install json: {err}")))?;
    println!("{encoded}");
    Ok(())
}

pub(super) fn print_titled_table(ui: &UiContext, title: &str, table: &comfy_table::Table) {
    let rendered_title = if ui.colors_enabled() {
        title.bold().underline().to_string()
    } else {
        title.to_string()
    };
    println!("  {rendered_title}");
    println!();
    print_indented_block(&table.to_string(), DRY_RUN_TABLE_INDENT);
}

pub(super) fn print_indented_block(content: &str, indent: usize) {
    let prefix = " ".repeat(indent);
    for line in content.lines() {
        println!("{prefix}{line}");
    }
}
