use std::path::Path;

use eden_skills_core::config::{config_dir_from_path, load_from_file, LoadOptions};
use eden_skills_core::error::EdenError;
use eden_skills_core::plan::{build_plan, Action, PlanItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommandOptions {
    pub strict: bool,
    pub json: bool,
}

pub fn plan(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path = Path::new(config_path);
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }

    let config_dir = config_dir_from_path(config_path);
    let plan = build_plan(&loaded.config, &config_dir)?;
    if options.json {
        print_plan_json_stub(&plan);
    } else {
        print_plan_text(&plan);
    }
    Ok(())
}

pub fn apply(config_path: &str, _options: CommandOptions) -> Result<(), EdenError> {
    println!("apply: pending implementation (config: {config_path})");
    Ok(())
}

pub fn doctor(config_path: &str, _options: CommandOptions) -> Result<(), EdenError> {
    println!("doctor: pending implementation (config: {config_path})");
    Ok(())
}

pub fn repair(config_path: &str, _options: CommandOptions) -> Result<(), EdenError> {
    println!("repair: pending implementation (config: {config_path})");
    Ok(())
}

fn print_plan_text(items: &[PlanItem]) {
    for item in items {
        println!(
            "{} {} {} -> {} ({})",
            action_label(item.action),
            item.skill_id,
            item.source_path,
            item.target_path,
            item.install_mode
        );
        for reason in &item.reasons {
            println!("  reason: {reason}");
        }
    }
}

fn print_plan_json_stub(items: &[PlanItem]) {
    println!("[");
    for (idx, item) in items.iter().enumerate() {
        let suffix = if idx + 1 == items.len() { "" } else { "," };
        println!(
            "  {{\"skill_id\":\"{}\",\"source_path\":\"{}\",\"target_path\":\"{}\",\"install_mode\":\"{}\",\"action\":\"{}\"}}{}",
            item.skill_id,
            item.source_path,
            item.target_path,
            item.install_mode,
            action_label(item.action),
            suffix
        );
    }
    println!("]");
}

fn action_label(action: Action) -> &'static str {
    match action {
        Action::Create => "create",
        Action::Update => "update",
        Action::Noop => "noop",
        Action::Conflict => "conflict",
    }
}
