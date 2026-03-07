use std::io::{IsTerminal, Write};
use std::process::ExitCode;
use std::sync::Once;

use clap::error::{ContextKind, ContextValue, Error as ClapError, ErrorKind as ClapErrorKind};
use eden_skills_cli::CliError;
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

/// Format and print a [`CliError`] using either the custom domain renderer or
/// the clap parse-error renderer.
///
/// Splits the error message at the `\nhint: ` convention, abbreviates
/// home-relative paths, and renders the hint with a purple `~>` prefix.
fn print_error(err: &CliError) {
    match err {
        CliError::Domain(err) => print_domain_error(err),
        CliError::Clap(err) => print_clap_error(err),
    }
}

fn print_domain_error(err: &EdenError) {
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

fn print_clap_error(err: &ClapError) {
    if let Some(rendered) = render_custom_clap_error(err) {
        if err.use_stderr() {
            eprint!("{rendered}");
        } else {
            print!("{rendered}");
        }
        return;
    }
    let _ = err.print();
}

fn render_custom_clap_error(err: &ClapError) -> Option<String> {
    match err.kind() {
        ClapErrorKind::InvalidSubcommand => Some(render_invalid_subcommand_error(err)),
        _ => None,
    }
}

fn render_invalid_subcommand_error(err: &ClapError) -> String {
    let colors_enabled = clap_error_colors_enabled(err);
    let invalid = clap_context_string(err, ContextKind::InvalidSubcommand)
        .unwrap_or_default()
        .to_string();
    let suggested = clap_context_strings(err, ContextKind::SuggestedSubcommand);
    let usage = clap_usage_text(err);

    let mut output = String::new();
    output.push_str(&style_error_prefix(colors_enabled));
    output.push(' ');
    output.push_str("unrecognized subcommand ");
    output.push_str(&style_quoted_token(&invalid, colors_enabled));

    if let Some(first_suggestion) = suggested.first() {
        output.push_str("\n\n  ");
        output.push_str(&style_tip_label(colors_enabled));
        output.push(' ');
        output.push_str("a similar subcommand exists: ");
        output.push_str(&style_quoted_token(first_suggestion, colors_enabled));
    }

    if let Some(usage) = usage {
        output.push_str("\n\n");
        output.push_str(&style_usage_line(&usage, colors_enabled));
    }

    output.push_str("\n\nFor more information, try ");
    output.push_str(&style_quoted_token("--help", colors_enabled));
    output.push_str(".\n");
    output
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

fn clap_context_string<'a>(err: &'a ClapError, kind: ContextKind) -> Option<&'a str> {
    match err.get(kind) {
        Some(ContextValue::String(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn clap_context_strings(err: &ClapError, kind: ContextKind) -> Vec<String> {
    match err.get(kind) {
        Some(ContextValue::Strings(values)) => values.clone(),
        Some(ContextValue::String(value)) => vec![value.clone()],
        _ => Vec::new(),
    }
}

fn clap_usage_text(err: &ClapError) -> Option<String> {
    match err.get(ContextKind::Usage) {
        Some(ContextValue::StyledStr(usage)) => Some(usage.to_string().trim_end().to_string()),
        Some(ContextValue::String(usage)) => Some(usage.trim_end().to_string()),
        _ => None,
    }
}

fn clap_error_colors_enabled(err: &ClapError) -> bool {
    let _ = err;
    eden_skills_cli::ui::color_output_enabled()
}

fn style_error_prefix(colors_enabled: bool) -> String {
    if colors_enabled {
        "error:".red().bold().to_string()
    } else {
        "error:".to_string()
    }
}

fn style_tip_label(colors_enabled: bool) -> String {
    if colors_enabled {
        "tip:".magenta().bold().to_string()
    } else {
        "tip:".to_string()
    }
}

fn style_usage_heading(colors_enabled: bool) -> String {
    if colors_enabled {
        "Usage:".green().bold().to_string()
    } else {
        "Usage:".to_string()
    }
}

fn style_quoted_token(token: &str, colors_enabled: bool) -> String {
    if colors_enabled {
        format!("{}", format!("'{token}'").cyan())
    } else {
        format!("'{token}'")
    }
}

fn style_usage_line(raw: &str, colors_enabled: bool) -> String {
    let mut lines = Vec::new();
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("Usage:") {
            let body = rest.trim_start();
            if body.is_empty() {
                lines.push(style_usage_heading(colors_enabled));
            } else {
                lines.push(format!(
                    "{} {}",
                    style_usage_heading(colors_enabled),
                    style_usage_body(body, colors_enabled)
                ));
            }
        } else {
            lines.push(style_usage_body(line, colors_enabled));
        }
    }
    lines.join("\n")
}

fn style_usage_body(raw: &str, colors_enabled: bool) -> String {
    if !colors_enabled {
        return raw.to_string();
    }

    let mut output = String::with_capacity(raw.len());
    let mut token = String::new();

    for ch in raw.chars() {
        if ch.is_whitespace() {
            if !token.is_empty() {
                output.push_str(&style_usage_token(&token));
                token.clear();
            }
            output.push(ch);
        } else {
            token.push(ch);
        }
    }

    if !token.is_empty() {
        output.push_str(&style_usage_token(&token));
    }

    output
}

fn style_usage_token(token: &str) -> String {
    if token.starts_with('[') && token.ends_with(']')
        || token.starts_with('<') && token.ends_with('>')
    {
        return token.magenta().to_string();
    }
    if token == "|" {
        return token.to_string();
    }
    token.cyan().to_string()
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
