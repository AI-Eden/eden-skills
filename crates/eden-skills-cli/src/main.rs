use std::io::{IsTerminal, Write};
use std::process::ExitCode;
use std::sync::Once;

use eden_skills_core::error::EdenError;
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() -> ExitCode {
    install_sigint_cursor_restore_handler();
    let result = eden_skills_cli::run().await;
    restore_terminal_cursor();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            print_error(&err);
            ExitCode::from(eden_skills_cli::exit_code_for_error(&err))
        }
    }
}

/// Format and print an [`EdenError`] to stderr.
///
/// Splits the error message at the `\nhint: ` convention, abbreviates
/// home-relative paths, and renders the hint with a purple `~>` prefix.
fn print_error(err: &EdenError) {
    let (message, hint) = user_message_and_hint(err);
    let message = abbreviate_message_paths(&message);
    let colors_enabled = eden_skills_cli::ui::color_output_enabled();
    let prefix = if colors_enabled {
        "error:".red().bold().to_string()
    } else {
        "error:".to_string()
    };
    eprintln!("{prefix} {message}");
    if let Some(hint) = hint {
        eprintln!();
        if colors_enabled {
            eprintln!("  {} {hint}", "~>".magenta());
        } else {
            eprintln!("  ~> {hint}");
        }
    }
}

fn user_message_and_hint(err: &EdenError) -> (String, Option<String>) {
    let raw = match err {
        EdenError::InvalidArguments(message) => message.clone(),
        EdenError::Validation(message) => format!("invalid config: {message}"),
        EdenError::Conflict(message) => message.clone(),
        EdenError::Runtime(message) => message.clone(),
        EdenError::Io(io) => match io.kind() {
            std::io::ErrorKind::PermissionDenied => format!(
                "permission denied: {io}\nhint: Check file permissions or run with appropriate privileges."
            ),
            _ => format!("io error: {io}"),
        },
    };
    split_hint(&raw)
}

fn split_hint(raw: &str) -> (String, Option<String>) {
    match raw.split_once("\nhint: ") {
        Some((message, hint)) => (message.to_string(), Some(hint.to_string())),
        None => (raw.to_string(), None),
    }
}

fn abbreviate_message_paths(message: &str) -> String {
    for prefix in [
        "config file not found: ",
        "permission denied reading config file: ",
        "storage directory not found: ",
        "permission denied writing to ",
    ] {
        if let Some(path) = message.strip_prefix(prefix) {
            return format!(
                "{prefix}{}",
                eden_skills_cli::ui::abbreviate_home_path(path)
            );
        }
    }
    message.to_string()
}

fn install_sigint_cursor_restore_handler() {
    static SETUP: Once = Once::new();
    SETUP.call_once(|| {
        let _ = ctrlc::set_handler(|| {
            if eden_skills_cli::signal::prompt_interruptible() {
                restore_terminal_cursor();
                eden_skills_cli::signal::request_prompt_interrupt();
                return;
            }
            let mut stderr = std::io::stderr();
            let _ = stderr.write_all(b"\n");
            let _ = stderr.flush();
            restore_terminal_cursor();
            std::process::exit(130);
        });
    });
}

fn restore_terminal_cursor() {
    if !(std::io::stdout().is_terminal() || std::io::stderr().is_terminal()) {
        return;
    }
    let mut stderr = std::io::stderr();
    let _ = stderr.write_all(b"\x1b[?25h");
    let _ = stderr.flush();
}
