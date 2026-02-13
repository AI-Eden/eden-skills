mod commands;

use std::env;
use std::process::ExitCode;

use commands::CommandOptions;
use eden_skills_core::error::EdenError;

const DEFAULT_CONFIG_PATH: &str = "~/.config/eden-skills/skills.yaml";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(exit_code_for_error(&err))
        }
    }
}

fn run() -> Result<(), EdenError> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        return Err(EdenError::InvalidArguments(
            "missing subcommand (plan|apply|doctor|repair)".to_string(),
        ));
    }

    let subcommand = args.remove(0);
    if matches!(subcommand.as_str(), "--help" | "-h" | "help") {
        print_usage();
        return Ok(());
    }

    let parsed = parse_global_options(&args)?;
    match subcommand.as_str() {
        "plan" => commands::plan(
            &parsed.config_path,
            CommandOptions {
                strict: parsed.strict,
                json: parsed.json,
            },
        ),
        "apply" => commands::apply(
            &parsed.config_path,
            CommandOptions {
                strict: parsed.strict,
                json: parsed.json,
            },
        ),
        "doctor" => commands::doctor(
            &parsed.config_path,
            CommandOptions {
                strict: parsed.strict,
                json: parsed.json,
            },
        ),
        "repair" => commands::repair(
            &parsed.config_path,
            CommandOptions {
                strict: parsed.strict,
                json: parsed.json,
            },
        ),
        _ => Err(EdenError::InvalidArguments(format!(
            "unsupported subcommand: {subcommand}"
        ))),
    }
}

struct ParsedGlobalOptions {
    config_path: String,
    strict: bool,
    json: bool,
}

fn parse_global_options(args: &[String]) -> Result<ParsedGlobalOptions, EdenError> {
    let mut parsed = ParsedGlobalOptions {
        config_path: DEFAULT_CONFIG_PATH.to_string(),
        strict: false,
        json: false,
    };

    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--config" => {
                let Some(value) = args.get(idx + 1) else {
                    return Err(EdenError::InvalidArguments(
                        "missing value for --config".to_string(),
                    ));
                };
                parsed.config_path = value.to_string();
                idx += 2;
            }
            "--strict" => {
                parsed.strict = true;
                idx += 1;
            }
            "--json" => {
                parsed.json = true;
                idx += 1;
            }
            unknown => {
                return Err(EdenError::InvalidArguments(format!(
                    "unsupported option: {unknown}"
                )));
            }
        }
    }
    Ok(parsed)
}

fn print_usage() {
    println!("eden-skills <plan|apply|doctor|repair> [--config <path>] [--strict] [--json]");
}

fn exit_code_for_error(err: &EdenError) -> u8 {
    match err {
        EdenError::InvalidArguments(_) | EdenError::Validation(_) => 2,
        EdenError::Conflict(_) => 3,
        EdenError::Runtime(_) | EdenError::Io(_) => 1,
    }
}
