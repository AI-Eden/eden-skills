pub mod commands;

use clap::{Args, Parser, Subcommand};
use commands::CommandOptions;
use eden_skills_core::config::InstallMode;
use eden_skills_core::error::EdenError;

pub const DEFAULT_CONFIG_PATH: &str = "~/.config/eden-skills/skills.toml";

pub async fn run() -> Result<(), EdenError> {
    run_with_args(std::env::args().skip(1).collect()).await
}

pub async fn run_with_args(args: Vec<String>) -> Result<(), EdenError> {
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
        Commands::Apply(args) => {
            commands::apply_async(
                &args.config,
                CommandOptions {
                    strict: args.strict,
                    json: args.json,
                },
                args.concurrency,
            )
            .await
        }
        Commands::Doctor(args) => commands::doctor(
            &args.config,
            CommandOptions {
                strict: args.strict,
                json: args.json,
            },
        ),
        Commands::Repair(args) => {
            commands::repair_async(
                &args.config,
                CommandOptions {
                    strict: args.strict,
                    json: args.json,
                },
                args.concurrency,
            )
            .await
        }
        Commands::Update(args) => {
            commands::update_async(commands::UpdateRequest {
                config_path: args.config,
                concurrency: args.concurrency,
                options: CommandOptions {
                    strict: args.strict,
                    json: args.json,
                },
            })
            .await
        }
        Commands::Install(args) => {
            commands::install_async(commands::InstallRequest {
                config_path: args.config,
                skill_name: args.skill_name,
                version: args.version,
                registry: args.registry,
                target: args.target,
                dry_run: args.dry_run,
                options: CommandOptions {
                    strict: args.strict,
                    json: args.json,
                },
            })
            .await
        }
        Commands::Init(args) => commands::init(&args.config, args.force),
        Commands::List(args) => commands::list(
            &args.config,
            CommandOptions {
                strict: args.strict,
                json: args.json,
            },
        ),
        Commands::Add(args) => commands::add(commands::AddRequest {
            config_path: args.config,
            id: args.id,
            repo: args.repo,
            r#ref: args.r#ref,
            subpath: args.subpath,
            mode: args.mode.into(),
            target_specs: args.target,
            verify_enabled: args.verify_enabled,
            verify_checks: args.verify_check,
            no_exec_metadata_only: args.no_exec_metadata_only,
            options: CommandOptions {
                strict: args.strict,
                json: args.json,
            },
        }),
        Commands::Remove(args) => {
            commands::remove_async(
                &args.config,
                &args.skill_id,
                CommandOptions {
                    strict: args.strict,
                    json: args.json,
                },
            )
            .await
        }
        Commands::Set(args) => commands::set(commands::SetRequest {
            config_path: args.config,
            skill_id: args.skill_id,
            repo: args.repo,
            r#ref: args.r#ref,
            subpath: args.subpath,
            mode: args.mode.map(Into::into),
            verify_enabled: args.verify_enabled,
            verify_checks: args.verify_check,
            target_specs: args.target,
            no_exec_metadata_only: args.no_exec_metadata_only,
            options: CommandOptions {
                strict: args.strict,
                json: args.json,
            },
        }),
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
    Apply(ApplyRepairArgs),
    Doctor(CommonArgs),
    Repair(ApplyRepairArgs),
    Update(UpdateArgs),
    Install(InstallArgs),
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
struct ApplyRepairArgs {
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,
    #[arg(long)]
    strict: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    concurrency: Option<usize>,
}

#[derive(Debug, Clone, Args)]
struct UpdateArgs {
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,
    #[arg(long)]
    strict: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    concurrency: Option<usize>,
}

#[derive(Debug, Clone, Args)]
struct InstallArgs {
    skill_name: String,

    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,
    #[arg(long)]
    strict: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    version: Option<String>,
    #[arg(long)]
    registry: Option<String>,
    #[arg(long)]
    target: Option<String>,
    #[arg(long)]
    dry_run: bool,
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
