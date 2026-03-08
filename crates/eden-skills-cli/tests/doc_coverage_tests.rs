/// TM-P28-003: Every `commands/` module starts with `//!`, `ui.rs` starts
/// with `//!`, core `lib.rs` starts with `//!`.
///
/// TM-P28-034: CLI module docs — all `commands/*.rs` have `//!`, all public
/// command functions have `///` doc comments.
///
/// TM-P28-035: Core module docs — listed core modules have `//!`, core
/// `lib.rs` has `//!` crate-level documentation.
///
/// TM-P28-036: UiContext documented — `ui.rs` has `//!`, public items
/// have `///` doc comments.

// ---------------------------------------------------------------------------
// TM-P28-003: Module doc comments present
// ---------------------------------------------------------------------------

#[test]
fn tm_p28_003_commands_modules_have_module_docs() {
    let cli_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands");
    let expected_modules: &[&str] = &[
        "mod.rs",
        "install/mod.rs",
        "reconcile.rs",
        "diagnose.rs",
        "plan_cmd.rs",
        "config_ops.rs",
        "remove.rs",
        "update.rs",
        "common.rs",
        "docker_cmd.rs",
        "clean.rs",
    ];
    for module_name in expected_modules {
        let path = cli_src.join(module_name);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{module_name} should be readable"));
        assert!(
            content.starts_with("//!"),
            "{module_name} must start with //! module doc"
        );
    }
}

#[test]
fn tm_p28_003_ui_has_module_doc() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/mod.rs");
    let content = std::fs::read_to_string(&path).expect("ui/mod.rs should be readable");
    assert!(
        content.starts_with("//!"),
        "ui/mod.rs must start with //! module doc"
    );
}

#[test]
fn tm_p28_003_core_lib_has_crate_doc() {
    let core_lib = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("eden-skills-core/src/lib.rs");
    let content = std::fs::read_to_string(&core_lib).expect("core lib.rs should be readable");
    assert!(
        content.starts_with("//!"),
        "core lib.rs must start with //! crate doc"
    );
}

// ---------------------------------------------------------------------------
// TM-P28-034: CLI module docs — public command functions documented
// ---------------------------------------------------------------------------

#[test]
fn tm_p28_034_public_command_functions_have_doc_comments() {
    let cli_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands");

    let checks: &[(&str, &[&str])] = &[
        ("install/mod.rs", &["pub async fn install_async"]),
        (
            "reconcile.rs",
            &["pub async fn apply_async", "pub async fn repair_async"],
        ),
        ("diagnose.rs", &["pub fn doctor"]),
        ("plan_cmd.rs", &["pub fn plan"]),
        (
            "config_ops.rs",
            &[
                "pub fn init",
                "pub fn list",
                "pub fn add",
                "pub fn set",
                "pub fn config_export",
                "pub fn config_import",
            ],
        ),
        ("remove.rs", &["pub async fn remove_many_async"]),
        ("update.rs", &["pub async fn update_async"]),
    ];

    for (file, functions) in checks {
        let path = cli_src.join(file);
        let content =
            std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{file} should be readable"));
        for func_sig in *functions {
            let sig_pos = content
                .find(func_sig)
                .unwrap_or_else(|| panic!("{file}: expected to find `{func_sig}`"));
            let before = &content[..sig_pos];
            assert!(
                before.trim_end().ends_with("///")
                    || before.contains(&format!("///\n{func_sig}"))
                    || has_doc_comment_before(before),
                "{file}: `{func_sig}` must be preceded by a /// doc comment"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// TM-P28-035: Core module docs
// ---------------------------------------------------------------------------

#[test]
fn tm_p28_035_core_modules_have_module_docs() {
    let core_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("eden-skills-core/src");

    let expected_modules: &[&str] = &[
        "reactor.rs",
        "lock.rs",
        "adapter/mod.rs",
        "source_format.rs",
        "discovery.rs",
        "config.rs",
        "plan.rs",
        "error.rs",
        "registry.rs",
        "source.rs",
        "paths.rs",
        "verify.rs",
        "safety.rs",
        "agents.rs",
        "state.rs",
        "managed.rs",
    ];

    for module_name in expected_modules {
        let path = core_src.join(module_name);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{module_name} should be readable"));
        assert!(
            content.starts_with("//!"),
            "core {module_name} must start with //! module doc"
        );
    }
}

// ---------------------------------------------------------------------------
// TM-P28-036: UiContext documented
// ---------------------------------------------------------------------------

#[test]
fn tm_p28_036_ui_public_items_have_doc_comments() {
    let ui_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui");

    let checks: &[(&str, &[&str])] = &[
        (
            "color.rs",
            &[
                "pub enum ColorWhen",
                "pub fn configure_color_output",
                "pub fn color_output_enabled",
            ],
        ),
        ("table.rs", &["pub enum StatusSymbol"]),
        (
            "context.rs",
            &["pub struct UiContext", "pub fn table(", "pub fn spinner("],
        ),
        (
            "format.rs",
            &[
                "pub struct UiSpinner",
                "pub fn abbreviate_home_path",
                "pub fn abbreviate_repo_url",
            ],
        ),
    ];

    for (file, items) in checks {
        let path = ui_dir.join(file);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("ui/{file} should be readable"));
        for item in *items {
            let pos = content
                .find(item)
                .unwrap_or_else(|| panic!("ui/{file}: expected to find `{item}`"));
            let before = &content[..pos];
            assert!(
                has_doc_comment_before(before),
                "ui/{file}: `{item}` must be preceded by a /// doc comment"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn has_doc_comment_before(text_before: &str) -> bool {
    let trimmed = text_before.trim_end();
    for line in trimmed.lines().rev() {
        let stripped = line.trim();
        if stripped.is_empty() {
            continue;
        }
        if stripped.starts_with("///") || stripped.starts_with("//!") {
            return true;
        }
        if stripped.starts_with("#[") || stripped.starts_with("//") {
            continue;
        }
        return false;
    }
    false
}
