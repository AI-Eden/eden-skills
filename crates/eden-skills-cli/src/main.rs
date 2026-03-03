use std::process::ExitCode;

use eden_skills_core::error::EdenError;
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() -> ExitCode {
    match eden_skills_cli::run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            print_error(&err);
            ExitCode::from(eden_skills_cli::exit_code_for_error(&err))
        }
    }
}

fn print_error(err: &EdenError) {
    let (message, hint) = user_message_and_hint(err);
    let colors_enabled = eden_skills_cli::ui::color_output_enabled();
    let prefix = if colors_enabled {
        "error:".red().bold().to_string()
    } else {
        "error:".to_string()
    };
    eprintln!("{prefix} {message}");
    eprintln!();
    if let Some(hint) = hint {
        if colors_enabled {
            eprint!(" {} ", "hint:".purple());
            eprintln!("{hint}");
        } else {
            eprintln!(" hint: {hint}");
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
