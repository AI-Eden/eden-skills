use std::io::{IsTerminal, Write};
use std::process::ExitCode;
use std::sync::Once;

use clap::error::{ContextKind, ContextValue, Error as ClapError, ErrorKind as ClapErrorKind};
use dialoguer::console::strip_ansi_codes;
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
        ClapErrorKind::ArgumentConflict => Some(render_argument_conflict_error(err)),
        ClapErrorKind::InvalidSubcommand => Some(render_invalid_subcommand_error(err)),
        ClapErrorKind::UnknownArgument => Some(render_unknown_argument_error(err)),
        ClapErrorKind::InvalidValue => Some(render_invalid_value_error(err)),
        ClapErrorKind::MissingRequiredArgument => Some(render_missing_required_argument_error(err)),
        ClapErrorKind::MissingSubcommand => Some(render_missing_subcommand_error(err)),
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
    let mut tips = Vec::new();

    if let Some(first_suggestion) = suggested.first() {
        tips.push(format!(
            "a similar subcommand exists: {}",
            style_quoted_cli_fragment(first_suggestion, colors_enabled)
        ));
    }
    tips.extend(clap_styled_suggestion_lines(err, colors_enabled));

    let mut output = render_parse_error_header(
        format!(
            "unrecognized subcommand {}",
            style_quoted_cli_fragment(&invalid, colors_enabled)
        ),
        colors_enabled,
    );
    append_tip_lines(&mut output, &tips, colors_enabled);
    append_usage_and_help(&mut output, usage.as_deref(), colors_enabled);
    output
}

fn render_argument_conflict_error(err: &ClapError) -> String {
    let colors_enabled = clap_error_colors_enabled(err);
    let usage = clap_usage_text(err);
    let invalid_arg = clap_context_string(err, ContextKind::InvalidArg).map(str::to_owned);
    let invalid_subcommand =
        clap_context_string(err, ContextKind::InvalidSubcommand).map(str::to_owned);
    let prior_args = clap_prior_args(err);

    let invalid = invalid_arg
        .as_deref()
        .or(invalid_subcommand.as_deref())
        .unwrap_or_default()
        .to_string();

    let message = if prior_args.len() == 1 && prior_args[0] == invalid {
        if invalid_arg.is_some() {
            format!(
                "the argument {} cannot be used multiple times",
                style_conflict_token(&invalid, colors_enabled)
            )
        } else {
            format!(
                "the subcommand {} cannot be used multiple times",
                style_conflict_token(&invalid, colors_enabled)
            )
        }
    } else if prior_args.len() == 1 {
        let subject = if invalid_arg.is_some() {
            "the argument"
        } else {
            "the subcommand"
        };
        format!(
            "{subject} {} cannot be used with {}",
            style_conflict_token(&invalid, colors_enabled),
            style_conflict_token(&prior_args[0], colors_enabled)
        )
    } else if !prior_args.is_empty() {
        let subject = if invalid_arg.is_some() {
            "the argument"
        } else {
            "the subcommand"
        };
        format!(
            "{subject} {} cannot be used with: {}",
            style_conflict_token(&invalid, colors_enabled),
            prior_args
                .iter()
                .map(|arg| style_conflict_token(arg, colors_enabled))
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else if invalid_arg.is_some() {
        format!(
            "the argument {} cannot be used here",
            style_conflict_token(&invalid, colors_enabled)
        )
    } else {
        format!(
            "the subcommand {} cannot be used here",
            style_conflict_token(&invalid, colors_enabled)
        )
    };

    let mut output = render_parse_error_header(message, colors_enabled);
    append_usage_and_help(&mut output, usage.as_deref(), colors_enabled);
    output
}

fn render_unknown_argument_error(err: &ClapError) -> String {
    let colors_enabled = clap_error_colors_enabled(err);
    let invalid = clap_context_string(err, ContextKind::InvalidArg)
        .unwrap_or_default()
        .to_string();
    let usage = clap_usage_text(err);
    let mut tips = Vec::new();

    if let Some(suggested_arg) = clap_context_string(err, ContextKind::SuggestedArg) {
        tips.push(format!(
            "a similar argument exists: {}",
            style_quoted_cli_fragment(suggested_arg, colors_enabled)
        ));
    }
    tips.extend(clap_styled_suggestion_lines(err, colors_enabled));

    let mut output = render_parse_error_header(
        format!(
            "unexpected argument {} found",
            style_unknown_argument_token(&invalid, colors_enabled)
        ),
        colors_enabled,
    );
    append_tip_lines(&mut output, &tips, colors_enabled);
    append_usage_and_help(&mut output, usage.as_deref(), colors_enabled);
    output
}

fn render_invalid_value_error(err: &ClapError) -> String {
    let colors_enabled = clap_error_colors_enabled(err);
    let invalid_arg = clap_context_string(err, ContextKind::InvalidArg)
        .unwrap_or_default()
        .to_string();
    let invalid_value = clap_context_string(err, ContextKind::InvalidValue)
        .unwrap_or_default()
        .to_string();
    let valid_values = clap_context_strings(err, ContextKind::ValidValue);
    let usage = clap_usage_text(err);
    let mut tips = Vec::new();

    if let Some(suggested_value) = clap_context_string(err, ContextKind::SuggestedValue) {
        tips.push(format!(
            "a similar value exists: {}",
            style_quoted_cli_fragment(suggested_value, colors_enabled)
        ));
    }

    let headline = if invalid_value.is_empty() {
        format!(
            "a value is required for {} but none was supplied",
            style_quoted_cli_fragment(&invalid_arg, colors_enabled)
        )
    } else {
        format!(
            "invalid value {} for {}",
            style_quoted_cli_fragment(&invalid_value, colors_enabled),
            style_quoted_cli_fragment(&invalid_arg, colors_enabled)
        )
    };

    let mut output = render_parse_error_header(headline, colors_enabled);
    if !valid_values.is_empty() {
        output.push_str("\n    [possible values: ");
        output.push_str(
            &valid_values
                .iter()
                .map(|value| style_quoted_cli_fragment(value, colors_enabled))
                .collect::<Vec<_>>()
                .join(", "),
        );
        output.push(']');
    }
    append_tip_lines(&mut output, &tips, colors_enabled);
    append_usage_and_help(&mut output, usage.as_deref(), colors_enabled);
    output
}

fn render_missing_required_argument_error(err: &ClapError) -> String {
    let colors_enabled = clap_error_colors_enabled(err);
    let missing = clap_context_strings(err, ContextKind::InvalidArg);
    let usage = clap_usage_text(err);

    let mut output = render_parse_error_header(
        "the following required arguments were not provided:".to_string(),
        colors_enabled,
    );
    for argument in missing {
        output.push_str("\n    ");
        output.push_str(&style_cli_fragment(&argument, colors_enabled));
    }
    append_usage_and_help(&mut output, usage.as_deref(), colors_enabled);
    output
}

fn render_missing_subcommand_error(err: &ClapError) -> String {
    let colors_enabled = clap_error_colors_enabled(err);
    let parent = clap_context_string(err, ContextKind::InvalidSubcommand)
        .unwrap_or_default()
        .to_string();
    let available = clap_context_strings(err, ContextKind::ValidSubcommand);
    let usage = clap_usage_text(err);
    let mut tips = Vec::new();

    if !available.is_empty() {
        tips.push(format!(
            "available subcommands: {}",
            available
                .iter()
                .map(|value| style_quoted_cli_fragment(value, colors_enabled))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let mut output = render_parse_error_header(
        format!(
            "{} requires a subcommand but one was not provided",
            style_quoted_cli_fragment(&parent, colors_enabled)
        ),
        colors_enabled,
    );
    append_tip_lines(&mut output, &tips, colors_enabled);
    append_usage_and_help(&mut output, usage.as_deref(), colors_enabled);
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

fn clap_context_string(err: &ClapError, kind: ContextKind) -> Option<&str> {
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

fn clap_prior_args(err: &ClapError) -> Vec<String> {
    clap_context_strings(err, ContextKind::PriorArg)
}

fn clap_context_styled_strings(err: &ClapError, kind: ContextKind) -> Vec<String> {
    match err.get(kind) {
        Some(ContextValue::StyledStrs(values)) => values
            .iter()
            .map(|value| strip_ansi_codes(&value.to_string()).to_string())
            .collect(),
        Some(ContextValue::StyledStr(value)) => {
            vec![strip_ansi_codes(&value.to_string()).to_string()]
        }
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

fn render_parse_error_header(message: String, colors_enabled: bool) -> String {
    format!("{} {message}", style_error_prefix(colors_enabled))
}

fn append_tip_lines(output: &mut String, tips: &[String], colors_enabled: bool) {
    if tips.is_empty() {
        return;
    }

    output.push_str("\n\n");
    for (index, tip) in tips.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push_str("  ");
        output.push_str(&style_tip_label(colors_enabled));
        output.push(' ');
        output.push_str(tip);
    }
}

fn append_usage_and_help(output: &mut String, usage: Option<&str>, colors_enabled: bool) {
    if let Some(usage) = usage {
        output.push_str("\n\n");
        output.push_str(&style_usage_line(usage, colors_enabled));
    }

    output.push_str("\n\nFor more information, try ");
    output.push_str(&style_quoted_cli_fragment("--help", colors_enabled));
    output.push_str(".\n");
}

fn clap_styled_suggestion_lines(err: &ClapError, colors_enabled: bool) -> Vec<String> {
    clap_context_styled_strings(err, ContextKind::Suggested)
        .into_iter()
        .map(|text| style_text_with_quoted_cli_fragments(&text, colors_enabled))
        .collect()
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
                    style_cli_fragment(body, colors_enabled)
                ));
            }
        } else {
            lines.push(style_cli_fragment(line, colors_enabled));
        }
    }
    lines.join("\n")
}

fn style_quoted_cli_fragment(fragment: &str, colors_enabled: bool) -> String {
    if !colors_enabled {
        return format!("'{fragment}'");
    }

    if !fragment.contains(char::is_whitespace)
        && !fragment.contains('<')
        && !fragment.contains('>')
        && !fragment.contains('[')
        && !fragment.contains(']')
    {
        return format!("{}", format!("'{fragment}'").cyan());
    }

    format!(
        "{}{}{}",
        "'".cyan(),
        style_cli_fragment(fragment, colors_enabled),
        "'".cyan()
    )
}

fn style_unknown_argument_token(token: &str, colors_enabled: bool) -> String {
    if !colors_enabled {
        return format!("'{token}'");
    }

    format!("'{}'", token.yellow())
}

fn style_conflict_token(token: &str, colors_enabled: bool) -> String {
    if !colors_enabled {
        return format!("'{token}'");
    }

    format!("'{}'", token.yellow())
}

fn style_cli_fragment(raw: &str, colors_enabled: bool) -> String {
    if !colors_enabled {
        return raw.to_string();
    }

    let mut output = String::with_capacity(raw.len());
    let mut token = String::new();

    for ch in raw.chars() {
        if ch.is_whitespace() {
            if !token.is_empty() {
                output.push_str(&style_cli_token(&token));
                token.clear();
            }
            output.push(ch);
        } else {
            token.push(ch);
        }
    }

    if !token.is_empty() {
        output.push_str(&style_cli_token(&token));
    }

    output
}

fn style_cli_token(token: &str) -> String {
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

fn style_text_with_quoted_cli_fragments(text: &str, colors_enabled: bool) -> String {
    if !colors_enabled {
        return text.to_string();
    }

    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('\'') {
        output.push_str(&rest[..start]);
        let quoted = &rest[start + 1..];
        let Some(end) = quoted.find('\'') else {
            output.push_str(&rest[start..]);
            return output;
        };
        output.push_str(&style_quoted_cli_fragment(&quoted[..end], colors_enabled));
        rest = &quoted[end + 1..];
    }
    output.push_str(rest);
    output
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
