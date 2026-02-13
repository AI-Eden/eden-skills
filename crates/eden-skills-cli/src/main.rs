mod commands;

use std::env;
use std::process::ExitCode;

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
    let config_path = parse_config_path(&args)?;

    match subcommand.as_str() {
        "plan" => commands::plan(&config_path),
        "apply" => commands::apply(&config_path),
        "doctor" => commands::doctor(&config_path),
        "repair" => commands::repair(&config_path),
        "--help" | "-h" | "help" => {
            print_usage();
            Ok(())
        }
        _ => Err(EdenError::InvalidArguments(format!(
            "unsupported subcommand: {subcommand}"
        ))),
    }
}

fn parse_config_path(args: &[String]) -> Result<String, EdenError> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--config" {
            let Some(value) = iter.next() else {
                return Err(EdenError::InvalidArguments(
                    "missing value for --config".to_string(),
                ));
            };
            return Ok(value.to_string());
        }
    }
    Ok(DEFAULT_CONFIG_PATH.to_string())
}

fn print_usage() {
    println!("eden-skills <plan|apply|doctor|repair> [--config <path>]");
}

fn exit_code_for_error(err: &EdenError) -> u8 {
    match err {
        EdenError::InvalidArguments(_) | EdenError::Validation(_) => 2,
        EdenError::Conflict(_) => 3,
        EdenError::Runtime(_) | EdenError::Io(_) => 1,
    }
}
