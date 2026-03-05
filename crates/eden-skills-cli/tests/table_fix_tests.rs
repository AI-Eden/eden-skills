use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use eden_skills_cli::ui::{configure_color_output, ColorWhen, UiContext};

#[test]
fn tm_p29_001_tty_table_factory_uses_content_driven_width() {
    let ui_source = read_source_file("src/ui.rs");
    assert!(
        ui_source.contains("ContentArrangement::Disabled"),
        "UiContext::table must use content-driven width (Disabled) in tty mode"
    );
    assert!(
        !ui_source.contains("ContentArrangement::DynamicFullWidth"),
        "UiContext::table must not force full-width table expansion in tty mode"
    );
    assert!(
        !ui_source.contains("(*header).bold().to_string()"),
        "table headers must remain plain text without ANSI styling"
    );
}

#[test]
fn tm_p29_002_non_tty_table_factory_keeps_dynamic_with_width_80() {
    let _guard = test_env_lock();
    std::env::remove_var("EDEN_SKILLS_FORCE_TTY");
    std::env::set_var("CI", "1");
    std::env::remove_var("NO_COLOR");
    std::env::remove_var("FORCE_COLOR");

    configure_color_output(ColorWhen::Auto, false);
    let ui = UiContext::from_env(false);
    let mut table = ui.table(&["Path", "Mode"]);
    table.add_row(vec!["x".repeat(200), "symlink".to_string()]);
    let rendered = table.to_string();

    assert!(
        rendered.contains('|'),
        "non-tty table should use ASCII borders, rendered={rendered}"
    );
    assert!(
        !rendered.contains('│') && !rendered.contains('┌'),
        "non-tty table must avoid UTF-8 borders, rendered={rendered}"
    );
    let max_width = rendered.lines().map(str::len).max().unwrap_or(0);
    assert!(
        max_width <= 80,
        "non-tty table must keep width fallback <= 80, got {max_width}, rendered={rendered}"
    );
}

#[test]
fn tm_p29_003_fixed_columns_apply_upper_boundary_constraints_at_call_sites() {
    let config_ops = read_source_file("src/commands/config_ops.rs");
    assert_contains_all(
        &config_ops,
        &[
            "ui.table(&[\"Skill\", \"Mode\", \"Source\", \"Agents\"])",
            "column_mut(1)",
            "Width::Fixed(8)",
        ],
        "config list table constraints",
    );

    let diagnose = read_source_file("src/commands/diagnose.rs");
    assert_contains_all(
        &diagnose,
        &[
            "ui.table(&[\"Sev\", \"Code\", \"Skill\"])",
            "column_mut(0)",
            "Width::Fixed(5)",
        ],
        "doctor summary table constraints",
    );

    let plan_cmd = read_source_file("src/commands/plan_cmd.rs");
    assert_contains_all(
        &plan_cmd,
        &[
            "ui.table(&[\"Action\", \"Skill\", \"Target\", \"Mode\"])",
            "column_mut(0)",
            "Width::Fixed(10)",
            "column_mut(3)",
            "Width::Fixed(8)",
        ],
        "plan table constraints",
    );

    let update = read_source_file("src/commands/update.rs");
    assert_contains_all(
        &update,
        &[
            "ui.table(&[\"Registry\", \"Status\", \"Detail\"])",
            "column_mut(1)",
            "Width::Fixed(10)",
        ],
        "update registry table constraints",
    );

    let install = read_source_file("src/commands/install.rs");
    assert!(
        !install.contains("ui.table(&[\"#\", \"Name\", \"Description\"])"),
        "install discovery preview should no longer use a table renderer in Phase 2.9"
    );
    assert_contains_all(
        &install,
        &[
            "ui.table(&[\"Agent\", \"Path\", \"Mode\"])",
            "column_mut(2)",
            "Width::Fixed(8)",
        ],
        "install --dry-run table constraints",
    );
}

#[test]
fn tm_p29_004_table_cells_use_plain_text_renderers() {
    let plan_cmd = read_source_file("src/commands/plan_cmd.rs");
    assert_contains_all(
        &plan_cmd,
        &[
            "table.add_row(vec![",
            "plan_action_cell(item.action)",
            "fn plan_action_cell(action: Action) -> String",
            "action_label(action).to_string()",
        ],
        "plan table action cells",
    );

    let update = read_source_file("src/commands/update.rs");
    assert_contains_all(
        &update,
        &[
            "registry_status_cell(&result.status)",
            "fn registry_status_cell(status: &RegistrySyncStatus) -> String",
            "status.as_str().to_string()",
        ],
        "update table status cells",
    );

    let diagnose = read_source_file("src/commands/diagnose.rs");
    assert_contains_all(
        &diagnose,
        &[
            "doctor_severity_cell(&finding.severity)",
            "fn doctor_severity_cell(severity: &str) -> String",
            "\"warn\".to_string()",
            "\"error\".to_string()",
        ],
        "doctor table severity cells",
    );
}

fn read_source_file(relative: &str) -> String {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = crate_root.join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn assert_contains_all(content: &str, fragments: &[&str], context: &str) {
    for fragment in fragments {
        assert!(
            content.contains(fragment),
            "{context} is missing required fragment `{fragment}`"
        );
    }
}

fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    match LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
