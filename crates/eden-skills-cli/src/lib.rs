pub mod commands;
pub mod ui;

use clap::{Args, Parser, Subcommand};
use commands::CommandOptions;
use eden_skills_core::config::InstallMode;
use eden_skills_core::error::EdenError;

pub const DEFAULT_CONFIG_PATH: &str = "~/.eden-skills/skills.toml";

pub async fn run() -> Result<(), EdenError> {
    run_with_args(std::env::args().skip(1).collect()).await
}

pub async fn run_with_args(args: Vec<String>) -> Result<(), EdenError> {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push("eden-skills".to_string());
    argv.extend(args);

    let cli = match Cli::try_parse_from(argv) {
        Ok(cli) => cli,
        Err(err) => match err.kind() {
            clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                err.print().map_err(EdenError::Io)?;
                return Ok(());
            }
            _ => return Err(EdenError::InvalidArguments(err.to_string())),
        },
    };

    match cli.command {
        Commands::Install(args) => {
            commands::install_async(commands::InstallRequest {
                config_path: args.config,
                source: args.source,
                id: args.id,
                r#ref: args.r#ref,
                skill: args.skill,
                all: args.all,
                yes: args.yes,
                list: args.list,
                version: args.version,
                registry: args.registry,
                target: args.target,
                dry_run: args.dry_run,
                copy: args.copy,
                options: CommandOptions {
                    strict: args.strict,
                    json: args.json,
                },
            })
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
        Commands::Remove(args) => {
            let _skip_confirmation = args.yes;
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
#[command(version)]
#[command(about = "Deterministic skill installation and reconciliation for agent environments")]
#[command(before_help = concat!("eden-skills ", env!("CARGO_PKG_VERSION")))]
#[command(
    long_about = "Deterministic skill installation and reconciliation for agent environments. eden-skills manages the full lifecycle of agent skills through a configuration-driven workflow. Use plan, apply, doctor, and repair to preview, reconcile, and validate installed state across targets."
)]
#[command(
    after_help = "Install & Update:\n  install   Install skills from a URL, path, or registry\n  update    Refresh registry sources\n  remove    Uninstall a skill and clean up its files\n\nState Reconciliation:\n  plan      Preview planned actions without making changes\n  apply     Reconcile installed state with configuration\n  doctor    Diagnose configuration and installation health\n  repair    Auto-repair drifted or broken installations\n\nConfiguration:\n  init      Create a new skills.toml configuration file\n  list      List configured skills and their targets\n  add       Add a skill entry to skills.toml\n  set       Modify properties of an existing skill entry\n  config    Export or import configuration\n\nExamples:\n  eden-skills install vercel-labs/agent-skills    Install skills from GitHub\n  eden-skills install ./my-local-skill            Install from local path\n  eden-skills list                                Show configured skills\n  eden-skills doctor                              Check installation health\n\nDocumentation: https://github.com/AI-Eden/eden-skills"
)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(
        about = "Install skills from a URL, path, or registry",
        next_help_heading = "Install & Update"
    )]
    Install(InstallArgs),
    #[command(
        about = "Refresh registry sources to latest versions",
        next_help_heading = "Install & Update"
    )]
    Update(UpdateArgs),
    #[command(
        about = "Uninstall a skill and clean up its files",
        next_help_heading = "Install & Update"
    )]
    Remove(RemoveArgs),
    #[command(
        about = "Preview planned actions without making changes",
        next_help_heading = "State Reconciliation"
    )]
    Plan(CommonArgs),
    #[command(
        about = "Reconcile installed state with configuration",
        next_help_heading = "State Reconciliation"
    )]
    Apply(ApplyRepairArgs),
    #[command(
        about = "Diagnose configuration and installation health",
        next_help_heading = "State Reconciliation"
    )]
    Doctor(CommonArgs),
    #[command(
        about = "Auto-repair drifted or broken installations",
        next_help_heading = "State Reconciliation"
    )]
    Repair(ApplyRepairArgs),
    #[command(
        about = "Create a new skills.toml configuration file",
        next_help_heading = "Configuration"
    )]
    Init(InitArgs),
    #[command(
        about = "List configured skills and their targets",
        next_help_heading = "Configuration"
    )]
    List(CommonArgs),
    #[command(
        about = "Add a skill entry to skills.toml",
        next_help_heading = "Configuration"
    )]
    Add(AddArgs),
    #[command(
        about = "Modify properties of an existing skill entry",
        next_help_heading = "Configuration"
    )]
    Set(SetArgs),
    #[command(
        about = "Export or import configuration",
        next_help_heading = "Configuration"
    )]
    Config(ConfigArgs),
}

#[derive(Debug, Clone, Args)]
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
enum ConfigSubcommand {
    #[command(about = "Export configuration to stdout")]
    Export(CommonArgs),
    #[command(about = "Import configuration from another file")]
    Import(ConfigImportArgs),
}

#[derive(Debug, Clone, Args)]
struct CommonArgs {
    #[arg(
        long,
        default_value = DEFAULT_CONFIG_PATH,
        hide_default_value = true,
        help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"
    )]
    config: String,
    #[arg(long, help = "Exit with error on drift or warnings")]
    strict: bool,
    #[arg(long, help = "Output machine-readable JSON")]
    json: bool,
}

#[derive(Debug, Clone, Args)]
struct ApplyRepairArgs {
    #[arg(
        long,
        default_value = DEFAULT_CONFIG_PATH,
        hide_default_value = true,
        help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"
    )]
    config: String,
    #[arg(long, help = "Exit with error on drift or warnings")]
    strict: bool,
    #[arg(long, help = "Output machine-readable JSON")]
    json: bool,
    #[arg(long, help = "Maximum number of concurrent operations")]
    concurrency: Option<usize>,
}

#[derive(Debug, Clone, Args)]
struct UpdateArgs {
    #[arg(
        long,
        default_value = DEFAULT_CONFIG_PATH,
        hide_default_value = true,
        help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"
    )]
    config: String,
    #[arg(long, help = "Exit with error on drift or warnings")]
    strict: bool,
    #[arg(long, help = "Output machine-readable JSON")]
    json: bool,
    #[arg(long, help = "Maximum number of concurrent operations")]
    concurrency: Option<usize>,
}

#[derive(Debug, Clone, Args)]
struct InstallArgs {
    #[arg(help = "URL, local path, or registry skill name")]
    source: String,

    #[arg(
        long,
        default_value = DEFAULT_CONFIG_PATH,
        hide_default_value = true,
        help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"
    )]
    config: String,
    #[arg(long, help = "Exit with error on drift or warnings")]
    strict: bool,
    #[arg(long, help = "Output machine-readable JSON")]
    json: bool,
    #[arg(long, help = "Version constraint for registry mode (e.g. >=1.0)")]
    version: Option<String>,
    #[arg(long, help = "Override the auto-derived skill identifier")]
    id: Option<String>,
    #[arg(long, help = "Git reference (branch, tag, or commit)")]
    r#ref: Option<String>,
    #[arg(
        short = 's',
        long,
        num_args = 1..,
        help = "Install only the named skill(s) from the repository"
    )]
    skill: Vec<String>,
    #[arg(long, help = "Install all discovered skills without confirmation")]
    all: bool,
    #[arg(short = 'y', long, help = "Skip all interactive confirmation prompts")]
    yes: bool,
    #[arg(long, help = "List discovered skills without installing")]
    list: bool,
    #[arg(long, help = "Use a specific registry for resolution")]
    registry: Option<String>,
    #[arg(
        short = 't',
        long,
        help = "Install to specific agent targets (e.g. claude-code, cursor)"
    )]
    target: Vec<String>,
    #[arg(long, help = "Preview what would be installed without making changes")]
    dry_run: bool,
    #[arg(long, help = "Use file copy instead of symlinks")]
    copy: bool,
}

#[derive(Debug, Clone, Args)]
struct ConfigImportArgs {
    #[arg(long, help = "Path to the source config file to import")]
    from: String,
    #[arg(
        long,
        default_value = DEFAULT_CONFIG_PATH,
        hide_default_value = true,
        help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"
    )]
    config: String,
    #[arg(long, help = "Preview import without writing changes")]
    dry_run: bool,
    #[arg(long, help = "Exit with error on drift or warnings")]
    strict: bool,
}

#[derive(Debug, Clone, Args)]
struct InitArgs {
    #[arg(
        long,
        default_value = DEFAULT_CONFIG_PATH,
        hide_default_value = true,
        help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"
    )]
    config: String,
    #[arg(long, help = "Overwrite existing config file")]
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
    #[arg(
        long,
        default_value = DEFAULT_CONFIG_PATH,
        hide_default_value = true,
        help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"
    )]
    config: String,
    #[arg(long, help = "Exit with error on drift or warnings")]
    strict: bool,
    #[arg(long, help = "Output machine-readable JSON")]
    json: bool,

    #[arg(long, help = "Unique skill identifier")]
    id: String,
    #[arg(long, help = "Source repository URL")]
    repo: String,
    #[arg(
        long,
        default_value = ".",
        hide_default_value = true,
        help = "Subdirectory within the repository [default: .]"
    )]
    subpath: String,
    #[arg(
        long,
        default_value = "main",
        hide_default_value = true,
        help = "Git reference [default: main]"
    )]
    r#ref: String,
    #[arg(
        long,
        value_enum,
        default_value = "symlink",
        hide_default_value = true,
        help = "Install mode: symlink or copy [default: symlink]"
    )]
    mode: InstallModeArg,

    #[arg(
        short = 't',
        long,
        required = true,
        num_args = 1..,
        help = "Agent targets (e.g. claude-code, cursor, custom:/path)"
    )]
    target: Vec<String>,

    #[arg(
        long,
        value_parser = clap::value_parser!(bool),
        help = "Enable post-install verification"
    )]
    verify_enabled: Option<bool>,
    #[arg(long, num_args = 1.., help = "Verification checks to run")]
    verify_check: Option<Vec<String>>,
    #[arg(
        long,
        value_parser = clap::value_parser!(bool),
        help = "Metadata-only mode (skip file installation)"
    )]
    no_exec_metadata_only: Option<bool>,
}

#[derive(Debug, Clone, Args)]
struct RemoveArgs {
    #[arg(
        long,
        default_value = DEFAULT_CONFIG_PATH,
        hide_default_value = true,
        help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"
    )]
    config: String,
    #[arg(long, help = "Exit with error on drift or warnings")]
    strict: bool,
    #[arg(long, help = "Output machine-readable JSON")]
    json: bool,
    #[arg(short = 'y', long, help = "Skip confirmation prompt")]
    yes: bool,

    #[arg(help = "One or more skill identifiers to remove")]
    skill_id: String,
}

#[derive(Debug, Clone, Args)]
struct SetArgs {
    #[arg(
        long,
        default_value = DEFAULT_CONFIG_PATH,
        hide_default_value = true,
        help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"
    )]
    config: String,
    #[arg(long, help = "Exit with error on drift or warnings")]
    strict: bool,
    #[arg(long, help = "Output machine-readable JSON")]
    json: bool,

    #[arg(help = "Skill identifier to modify")]
    skill_id: String,

    #[arg(long, help = "New source repository URL")]
    repo: Option<String>,
    #[arg(long, help = "New subdirectory within the repository")]
    subpath: Option<String>,
    #[arg(long, help = "New Git reference")]
    r#ref: Option<String>,
    #[arg(long, value_enum, help = "New install mode: symlink or copy")]
    mode: Option<InstallModeArg>,
    #[arg(
        long,
        value_parser = clap::value_parser!(bool),
        help = "Enable or disable verification"
    )]
    verify_enabled: Option<bool>,
    #[arg(long, num_args = 1.., help = "Replace verification checks")]
    verify_check: Option<Vec<String>>,
    #[arg(short = 't', long, num_args = 1.., help = "Replace all targets")]
    target: Option<Vec<String>>,
    #[arg(
        long,
        value_parser = clap::value_parser!(bool),
        help = "Set metadata-only mode"
    )]
    no_exec_metadata_only: Option<bool>,
}
