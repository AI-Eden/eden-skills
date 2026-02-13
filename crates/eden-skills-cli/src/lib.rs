pub mod commands;

use std::env;

use commands::CommandOptions;
use eden_skills_core::error::EdenError;

pub const DEFAULT_CONFIG_PATH: &str = "~/.config/eden-skills/skills.toml";

pub fn run() -> Result<(), EdenError> {
    run_with_args(env::args().skip(1).collect())
}

pub fn run_with_args(args: Vec<String>) -> Result<(), EdenError> {
    let Some((subcommand, option_args)) = args.split_first() else {
        print_usage();
        return Err(EdenError::InvalidArguments(
            "missing subcommand (plan|apply|doctor|repair)".to_string(),
        ));
    };

    if matches!(subcommand.as_str(), "--help" | "-h" | "help") {
        print_usage();
        return Ok(());
    }

    let parsed = parse_global_options(option_args)?;
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

pub fn exit_code_for_error(err: &EdenError) -> u8 {
    match err {
        EdenError::InvalidArguments(_) | EdenError::Validation(_) => 2,
        EdenError::Conflict(_) => 3,
        EdenError::Runtime(_) | EdenError::Io(_) => 1,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    let mut remaining = args;
    while let Some((arg, tail)) = remaining.split_first() {
        match arg.as_str() {
            "--config" => {
                let Some((value, after_value)) = tail.split_first() else {
                    return Err(EdenError::InvalidArguments(
                        "missing value for --config".to_string(),
                    ));
                };
                parsed.config_path = value.clone();
                remaining = after_value;
            }
            "--strict" => {
                parsed.strict = true;
                remaining = tail;
            }
            "--json" => {
                parsed.json = true;
                remaining = tail;
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

#[cfg(test)]
mod tests {
    use super::parse_global_options;

    fn args(input: &[&str]) -> Vec<String> {
        input.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_global_options_defaults() {
        let parsed = parse_global_options(&args(&[])).expect("parse options");
        assert!(!parsed.strict);
        assert!(!parsed.json);
        assert!(parsed.config_path.ends_with("skills.toml"));
    }

    #[test]
    fn parse_global_options_with_flags_and_config() {
        let parsed =
            parse_global_options(&args(&["--strict", "--config", "./custom.toml", "--json"]))
                .expect("parse options");

        assert!(parsed.strict);
        assert!(parsed.json);
        assert_eq!(parsed.config_path, "./custom.toml");
    }

    #[test]
    fn parse_global_options_missing_config_value() {
        let err =
            parse_global_options(&args(&["--config"])).expect_err("expected missing value error");
        let message = err.to_string();
        assert!(message.contains("missing value for --config"));
    }
}
