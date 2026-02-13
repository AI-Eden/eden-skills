pub mod commands;

use clap::{Args, Parser, Subcommand};
use commands::CommandOptions;
use eden_skills_core::config::InstallMode;
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
        Commands::Add(args) => commands::add(
            &args.config,
            &args.id,
            &args.repo,
            &args.r#ref,
            &args.subpath,
            args.mode.into(),
            &args.target,
            args.verify_enabled,
            args.verify_check.as_deref(),
            args.no_exec_metadata_only,
            CommandOptions {
                strict: args.strict,
                json: args.json,
            },
        ),
        Commands::Remove(args) => commands::remove(
            &args.config,
            &args.skill_id,
            CommandOptions {
                strict: args.strict,
                json: args.json,
            },
        ),
        Commands::Set(args) => commands::set(
            &args.config,
            &args.skill_id,
            args.repo.as_deref(),
            args.r#ref.as_deref(),
            args.subpath.as_deref(),
            args.mode.map(Into::into),
            args.verify_enabled,
            args.verify_check.as_deref(),
            args.target.as_deref(),
            args.no_exec_metadata_only,
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
            ConfigSubcommand::Import(args) => commands::config_import(
                &args.from,
                &args.config,
                args.dry_run,
                CommandOptions {
                    strict: args.strict,
                    json: false,
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
    Add(AddArgs),
    Remove(RemoveArgs),
    Set(SetArgs),
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
    Import(ConfigImportArgs),
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
struct ConfigImportArgs {
    #[arg(long)]
    from: String,
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    strict: bool,
}

#[derive(Debug, Clone, Args)]
struct InitArgs {
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum InstallModeArg {
    Symlink,
    Copy,
}

impl From<InstallModeArg> for InstallMode {
    fn from(value: InstallModeArg) -> Self {
        match value {
            InstallModeArg::Symlink => InstallMode::Symlink,
            InstallModeArg::Copy => InstallMode::Copy,
        }
    }
}

#[derive(Debug, Clone, Args)]
struct AddArgs {
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,
    #[arg(long)]
    strict: bool,
    #[arg(long)]
    json: bool,

    #[arg(long)]
    id: String,
    #[arg(long)]
    repo: String,
    #[arg(long, default_value = ".")]
    subpath: String,
    #[arg(long, default_value = "main")]
    r#ref: String,
    #[arg(long, value_enum, default_value = "symlink")]
    mode: InstallModeArg,

    #[arg(long, required = true, num_args = 1..)]
    target: Vec<String>,

    #[arg(long, value_parser = clap::value_parser!(bool))]
    verify_enabled: Option<bool>,
    #[arg(long, num_args = 1..)]
    verify_check: Option<Vec<String>>,
    #[arg(long, value_parser = clap::value_parser!(bool))]
    no_exec_metadata_only: Option<bool>,
}

#[derive(Debug, Clone, Args)]
struct RemoveArgs {
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,
    #[arg(long)]
    strict: bool,
    #[arg(long)]
    json: bool,

    skill_id: String,
}

#[derive(Debug, Clone, Args)]
struct SetArgs {
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,
    #[arg(long)]
    strict: bool,
    #[arg(long)]
    json: bool,

    skill_id: String,

    #[arg(long)]
    repo: Option<String>,
    #[arg(long)]
    subpath: Option<String>,
    #[arg(long)]
    r#ref: Option<String>,
    #[arg(long, value_enum)]
    mode: Option<InstallModeArg>,
    #[arg(long, value_parser = clap::value_parser!(bool))]
    verify_enabled: Option<bool>,
    #[arg(long, num_args = 1..)]
    verify_check: Option<Vec<String>>,
    #[arg(long, num_args = 1..)]
    target: Option<Vec<String>>,
    #[arg(long, value_parser = clap::value_parser!(bool))]
    no_exec_metadata_only: Option<bool>,
}
