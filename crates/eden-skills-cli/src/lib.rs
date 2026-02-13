pub mod commands;

use clap::{Args, Parser, Subcommand};
use commands::CommandOptions;
use eden_skills_core::error::EdenError;

pub const DEFAULT_CONFIG_PATH: &str = "~/.config/eden-skills/skills.toml";

pub fn run() -> Result<(), EdenError> {
    run_with_args(std::env::args().skip(1).collect())
}

pub fn run_with_args(args: Vec<String>) -> Result<(), EdenError> {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push("eden-skills".to_string());
    argv.extend(args);

    let cli =
        Cli::try_parse_from(argv).map_err(|err| EdenError::InvalidArguments(err.to_string()))?;

    match cli.command {
        Commands::Plan(args) => commands::plan(
            &args.config,
            CommandOptions {
                strict: args.strict,
                json: args.json,
            },
        ),
        Commands::Apply(args) => commands::apply(
            &args.config,
            CommandOptions {
                strict: args.strict,
                json: args.json,
            },
        ),
        Commands::Doctor(args) => commands::doctor(
            &args.config,
            CommandOptions {
                strict: args.strict,
                json: args.json,
            },
        ),
        Commands::Repair(args) => commands::repair(
            &args.config,
            CommandOptions {
                strict: args.strict,
                json: args.json,
            },
        ),
        Commands::Init(args) => commands::init(&args.config, args.force),
        Commands::List(args) => commands::list(
            &args.config,
            CommandOptions {
                strict: args.strict,
                json: args.json,
            },
        ),
        Commands::Config(cmd) => match cmd.command {
            ConfigSubcommand::Export(args) => commands::config_export(
                &args.config,
                CommandOptions {
                    strict: args.strict,
                    json: args.json,
                },
            ),
        },
    }
}

pub fn exit_code_for_error(err: &EdenError) -> u8 {
    match err {
        EdenError::InvalidArguments(_) | EdenError::Validation(_) => 2,
        EdenError::Conflict(_) => 3,
        EdenError::Runtime(_) | EdenError::Io(_) => 1,
    }
}

#[derive(Debug, Parser)]
#[command(name = "eden-skills")]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Plan(CommonArgs),
    Apply(CommonArgs),
    Doctor(CommonArgs),
    Repair(CommonArgs),
    Init(InitArgs),
    List(CommonArgs),
    Config(ConfigArgs),
}

#[derive(Debug, Clone, Args)]
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
enum ConfigSubcommand {
    Export(CommonArgs),
}

#[derive(Debug, Clone, Args)]
struct CommonArgs {
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,
    #[arg(long)]
    strict: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Args)]
struct InitArgs {
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,
    #[arg(long)]
    force: bool,
}
