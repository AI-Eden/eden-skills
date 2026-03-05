use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use eden_skills_cli::ui::{
    abbreviate_home_path, abbreviate_repo_url, configure_color_output, ColorWhen, UiContext,
};

#[test]
fn comfy_table_dependency_is_declared_in_cli_cargo_toml() {
    let cargo_toml = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read crates/eden-skills-cli/Cargo.toml");
    assert!(
        cargo_toml
            .lines()
            .any(|line| line.trim_start().starts_with("comfy-table")),
        "comfy-table must be declared as a direct dependency"
    );
}

#[test]
fn ui_context_table_uses_utf8_borders_plain_headers_and_content_driven_width_on_tty() {
    let _guard = test_env_lock();
    std::env::set_var("EDEN_SKILLS_FORCE_TTY", "1");
    std::env::remove_var("CI");
    std::env::remove_var("NO_COLOR");
    std::env::remove_var("FORCE_COLOR");

    configure_color_output(ColorWhen::Always, false);
    let ui = UiContext::from_env(false);
    let mut table = ui.table(&["Skill", "Mode"]);
    table.add_row(vec!["demo-skill", "copy"]);

    let rendered = table.to_string();
    assert!(
        rendered.contains('│') || rendered.contains('┌'),
        "TTY table should use UTF-8 borders, rendered={rendered}"
    );
    assert!(
        !has_ansi_codes(&rendered),
        "TTY table must not include ANSI styling in headers or cells, rendered={rendered}"
    );
    let max_width = rendered
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    assert!(
        max_width < 60,
        "TTY table should size to content instead of terminal width, got {max_width}, rendered={rendered}"
    );
}

#[test]
fn ui_context_table_uses_ascii_borders_and_wraps_to_80_on_non_tty() {
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
        "non-TTY table should use ASCII vertical borders, rendered={rendered}"
    );
    assert!(
        !rendered.contains('│') && !rendered.contains('┌'),
        "non-TTY table must avoid UTF-8 borders, rendered={rendered}"
    );
    assert!(
        !has_ansi_codes(&rendered),
        "non-TTY table should not include ANSI styling by default, rendered={rendered}"
    );
    let max_width = rendered.lines().map(str::len).max().unwrap_or(0);
    assert!(
        max_width <= 80,
        "non-TTY table must wrap to fallback width <= 80, got {max_width}, rendered={rendered}"
    );
}

#[test]
fn abbreviate_home_path_replaces_home_prefix_and_preserves_non_home_paths() {
    let home = resolve_home_for_test();
    let home_trimmed = home.trim_end_matches(['/', '\\']);
    let sep = if home_trimmed.contains('\\') {
        '\\'
    } else {
        '/'
    };

    let under_home = format!("{home_trimmed}{sep}.claude{sep}skills{sep}x");
    let expected = "~/.claude/skills/x".to_string();
    assert_eq!(abbreviate_home_path(&under_home), expected);

    let exact_home = home_trimmed.to_string();
    assert_eq!(abbreviate_home_path(&exact_home), "~");

    let outside_home = format!("{home_trimmed}__outside{sep}skills{sep}x");
    assert_eq!(abbreviate_home_path(&outside_home), outside_home);
}

#[test]
fn abbreviate_repo_url_extracts_github_owner_and_repo() {
    assert_eq!(
        abbreviate_repo_url("https://github.com/owner/repo.git"),
        "owner/repo"
    );
    assert_eq!(
        abbreviate_repo_url("https://github.com/owner/repo"),
        "owner/repo"
    );
    assert_eq!(
        abbreviate_repo_url("https://example.com/owner/repo.git"),
        "https://example.com/owner/repo.git"
    );
}

fn has_ansi_codes(text: &str) -> bool {
    text.as_bytes().windows(2).any(|window| window == b"\x1b[")
}

fn resolve_home_for_test() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("HOME or USERPROFILE must be set for abbreviation tests")
}

fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    match LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
