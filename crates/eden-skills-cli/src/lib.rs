//! eden-skills CLI binary crate.
//!
//! Parses command-line arguments via `clap`, configures color output,
//! and dispatches to the appropriate command in [`commands`]. The
//! [`ui`] module provides terminal-aware rendering primitives used by
//! every command's human-mode output path.

pub mod commands;
pub mod signal;
pub mod ui;

use std::io::IsTerminal;

use clap::builder::styling::{AnsiColor, Style, Styles};
use clap::builder::StyledStr;
use clap::{Args, ColorChoice, FromArgMatches, Parser, Subcommand};
use commands::CommandOptions;
use eden_skills_core::config::InstallMode;
use eden_skills_core::error::EdenError;
use ui::{configure_color_output, ColorWhen};

pub const DEFAULT_CONFIG_PATH: &str = "~/.eden-skills/skills.toml";

/// Top-level CLI error preserving either domain failures or clap parse errors.
#[derive(Debug)]
pub enum CliError {
    /// An eden-skills domain or runtime failure after argument parsing succeeded.
    Domain(EdenError),
    /// A clap parsing / help-rendering error emitted before command dispatch.
    Clap(clap::Error),
}

impl From<EdenError> for CliError {
    fn from(value: EdenError) -> Self {
        Self::Domain(value)
    }
}

impl From<clap::Error> for CliError {
    fn from(value: clap::Error) -> Self {
        Self::Clap(value)
    }
}

pub async fn run() -> Result<(), CliError> {
    run_with_args(std::env::args().skip(1).collect()).await
}

pub async fn run_with_args(args: Vec<String>) -> Result<(), CliError> {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push("eden-skills".to_string());
    argv.extend(args);
    let clap_color_choice = resolve_clap_color_choice(&argv);
    configure_color_output(color_when_from_clap_choice(clap_color_choice), false);

    if argv.len() == 1 {
        let _ = build_cli_command(clap_color_choice).print_help();
        println!();
        return Ok(());
    }

    let matches = match build_cli_command(clap_color_choice).try_get_matches_from(&argv) {
        Ok(matches) => matches,
        Err(err) => match err.kind() {
            clap::error::ErrorKind::DisplayHelp
            | clap::error::ErrorKind::DisplayVersion
            | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                err.print().map_err(EdenError::Io).map_err(CliError::from)?;
                return Ok(());
            }
            _ => return Err(CliError::from(err)),
        },
    };
    let cli = Cli::from_arg_matches(&matches).map_err(CliError::from)?;
    configure_color_output(cli.color, cli.command.json_mode());

    let result = match cli.command {
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
                force: args.force,
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
                apply: args.apply,
                options: CommandOptions {
                    strict: args.strict,
                    json: args.json,
                },
            })
            .await
        }
        Commands::Remove(args) => {
            commands::remove_many_async(
                &args.config,
                &args.skill_ids,
                args.yes,
                args.force,
                args.auto_clean,
                CommandOptions {
                    strict: args.strict,
                    json: args.json,
                },
            )
            .await
        }
        Commands::Clean(args) => commands::clean(
            &args.config,
            args.dry_run,
            CommandOptions {
                strict: false,
                json: args.json,
            },
        ),
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
                args.force,
            )
            .await
        }
        Commands::Doctor(args) => commands::doctor(
            &args.config,
            CommandOptions {
                strict: args.strict,
                json: args.json,
            },
            args.no_warning,
        ),
        Commands::Docker(args) => match args.command {
            DockerSubcommand::MountHint(cmd) => {
                commands::docker_mount_hint_async(&cmd.container, &cmd.config).await
            }
        },
        Commands::Repair(args) => {
            commands::repair_async(
                &args.config,
                CommandOptions {
                    strict: args.strict,
                    json: args.json,
                },
                args.concurrency,
                args.force,
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
    };
    result.map_err(CliError::from)
}

pub fn exit_code_for_error(err: &CliError) -> u8 {
    match err {
        CliError::Domain(EdenError::InvalidArguments(_))
        | CliError::Domain(EdenError::Validation(_)) => 2,
        CliError::Domain(EdenError::Conflict(_)) => 3,
        CliError::Domain(EdenError::Runtime(_) | EdenError::Io(_)) => 1,
        CliError::Clap(err) => err.exit_code() as u8,
    }
}

const DOCS_URL: &str = "https://github.com/AI-Eden/eden-skills/blob/main/README.md";

#[derive(Debug, Clone, Copy)]
struct HelpExample {
    subcommand: &'static str,
    argument: Option<&'static str>,
    description: &'static str,
}

impl HelpExample {
    fn plain_command(self) -> String {
        match self.argument {
            Some(argument) => format!("eden-skills {} {}", self.subcommand, argument),
            None => format!("eden-skills {}", self.subcommand),
        }
    }
}

const ROOT_HELP_EXAMPLES: &[HelpExample] = &[
    HelpExample {
        subcommand: "install",
        argument: Some("vercel-labs/agent-skills"),
        description: "Install skills from GitHub",
    },
    HelpExample {
        subcommand: "install",
        argument: Some("./my-local-skill"),
        description: "Install from local path",
    },
    HelpExample {
        subcommand: "list",
        argument: None,
        description: "Show configured skills",
    },
    HelpExample {
        subcommand: "doctor",
        argument: None,
        description: "Check installation health",
    },
];

fn help_styles() -> Styles {
    Styles::styled()
        .header(Style::new().bold().fg_color(Some(AnsiColor::Green.into())))
        .literal(Style::new().bold().fg_color(Some(AnsiColor::Cyan.into())))
        .placeholder(Style::new().fg_color(Some(AnsiColor::Magenta.into())))
        .usage(Style::new().bold().fg_color(Some(AnsiColor::Green.into())))
}

fn build_cli_command(clap_color_choice: ColorChoice) -> clap::Command {
    <Cli as clap::CommandFactory>::command()
        .color(clap_color_choice)
        .after_help(render_root_help_footer(clap_color_choice))
}

fn render_root_help_footer(clap_color_choice: ColorChoice) -> StyledStr {
    let styles = help_styles();
    let colors_enabled = help_colors_enabled(clap_color_choice);
    let command_width = ROOT_HELP_EXAMPLES
        .iter()
        .map(|example| example.plain_command().len())
        .max()
        .unwrap_or_default();

    let mut footer = StyledStr::new();
    push_help_span(
        &mut footer,
        "Examples:",
        styles.get_header(),
        colors_enabled,
    );
    footer.push_str("\n");

    for example in ROOT_HELP_EXAMPLES {
        footer.push_str("  ");
        push_help_span(
            &mut footer,
            "eden-skills",
            styles.get_literal(),
            colors_enabled,
        );
        footer.push_str(" ");
        push_help_span(
            &mut footer,
            example.subcommand,
            styles.get_literal(),
            colors_enabled,
        );
        if let Some(argument) = example.argument {
            footer.push_str(" ");
            push_help_span(
                &mut footer,
                argument,
                styles.get_placeholder(),
                colors_enabled,
            );
        }

        let padding = command_width
            .saturating_sub(example.plain_command().len())
            .saturating_add(4);
        footer.push_str(&" ".repeat(padding));
        footer.push_str(example.description);
        footer.push_str("\n");
    }

    footer.push_str("\n");
    push_help_span(
        &mut footer,
        "Documentation:",
        styles.get_header(),
        colors_enabled,
    );
    footer.push_str(" ");
    push_help_span(
        &mut footer,
        DOCS_URL,
        styles.get_placeholder(),
        colors_enabled,
    );
    footer
}

fn push_help_span(buffer: &mut StyledStr, text: &str, style: &Style, colors_enabled: bool) {
    if !colors_enabled {
        buffer.push_str(text);
        return;
    }

    buffer.push_str(&format!("{}{text}{}", style.render(), style.render_reset()));
}

fn help_colors_enabled(clap_color_choice: ColorChoice) -> bool {
    match clap_color_choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => {
            if env_var_present("NO_COLOR") {
                return false;
            }
            if env_var_present("FORCE_COLOR") {
                return true;
            }
            if env_var_present("CI") {
                return false;
            }
            std::env::var("EDEN_SKILLS_FORCE_TTY")
                .ok()
                .as_deref()
                .is_some_and(|value| value != "0" && !value.is_empty())
                || std::io::stdout().is_terminal()
        }
    }
}

fn env_var_present(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty())
}

fn resolve_clap_color_choice(argv: &[String]) -> ColorChoice {
    let mut next_is_color_value = false;
    let mut parsed = ColorChoice::Auto;
    for arg in argv.iter().skip(1) {
        if next_is_color_value {
            parsed = parse_clap_color_choice(arg);
            next_is_color_value = false;
            continue;
        }
        if arg == "--color" {
            next_is_color_value = true;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--color=") {
            parsed = parse_clap_color_choice(value);
        }
    }
    parsed
}

fn parse_clap_color_choice(value: &str) -> ColorChoice {
    match value {
        "always" => ColorChoice::Always,
        "never" => ColorChoice::Never,
        _ => ColorChoice::Auto,
    }
}

fn color_when_from_clap_choice(choice: ColorChoice) -> ColorWhen {
    match choice {
        ColorChoice::Always => ColorWhen::Always,
        ColorChoice::Never => ColorWhen::Never,
        ColorChoice::Auto => ColorWhen::Auto,
    }
}

#[derive(Debug, Parser)]
#[command(name = "eden-skills")]
#[command(version)]
#[command(about = "Deterministic & Blazing-Fast Skills Manager for AI Agents.")]
#[command(before_help = concat!("eden-skills ", env!("CARGO_PKG_VERSION")))]
#[command(styles = help_styles())]
#[command(
    long_about = "Deterministic & Blazing-Fast Skills Manager for AI Agents (Claude Code, Cursor, Codex & More)."
)]
struct Cli {
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = ColorWhen::Auto,
        help = "Control color output"
    )]
    color: ColorWhen,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(
        about = "Install skills from a URL, path, or registry",
        next_help_heading = "Quick Management"
    )]
    Install(InstallArgs),
    #[command(
        about = "Refresh registry sources to latest versions",
        next_help_heading = "Quick Management"
    )]
    Update(UpdateArgs),
    #[command(
        about = "Uninstall a skill and clean up its files",
        next_help_heading = "Quick Management"
    )]
    Remove(RemoveArgs),
    #[command(
        about = "Remove orphaned cache entries and stale discovery directories",
        next_help_heading = "State Reconciliation"
    )]
    Clean(CleanArgs),
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
    Doctor(DoctorArgs),
    #[command(
        about = "Docker-specific utilities",
        next_help_heading = "Container Utilities"
    )]
    Docker(DockerArgs),
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

impl Commands {
    fn json_mode(&self) -> bool {
        match self {
            Self::Install(args) => args.json,
            Self::Update(args) => args.json,
            Self::Remove(args) => args.json,
            Self::Clean(args) => args.json,
            Self::Plan(args) => args.json,
            Self::Apply(args) => args.json,
            Self::Doctor(args) => args.json,
            Self::Docker(_) => false,
            Self::Repair(args) => args.json,
            Self::Init(_) => false,
            Self::List(args) => args.json,
            Self::Add(args) => args.json,
            Self::Set(args) => args.json,
            Self::Config(args) => match &args.command {
                ConfigSubcommand::Export(export) => export.json,
                ConfigSubcommand::Import(_) => false,
            },
        }
    }
}

#[derive(Debug, Clone, Args)]
struct DockerArgs {
    #[command(subcommand)]
    command: DockerSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
enum DockerSubcommand {
    #[command(about = "Print recommended bind mounts for a container")]
    MountHint(DockerMountHintArgs),
}

#[derive(Debug, Clone, Args)]
struct DockerMountHintArgs {
    #[arg(help = "Running Docker container name")]
    container: String,
    #[arg(
        long,
        default_value = DEFAULT_CONFIG_PATH,
        hide_default_value = true,
        help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"
    )]
    config: String,
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
struct DoctorArgs {
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
    #[arg(long, help = "Hide warning-level findings from output")]
    no_warning: bool,
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
    #[arg(long, help = "Force ownership reclaim for docker-managed targets")]
    force: bool,
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
    #[arg(
        long,
        help = "After refresh, reconcile skills with detected source updates"
    )]
    apply: bool,
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
    #[arg(long, help = "Overwrite externally-managed targets and take ownership")]
    force: bool,
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
    #[arg(
        long,
        help = "Remove files even when the manifest marks them as externally managed"
    )]
    force: bool,
    #[arg(long, help = "Run cache cleanup after removing the selected skills")]
    auto_clean: bool,

    #[arg(
        value_name = "SKILL_ID",
        help = "One or more skill identifiers to remove"
    )]
    skill_ids: Vec<String>,
}

#[derive(Debug, Clone, Args)]
struct CleanArgs {
    #[arg(
        long,
        default_value = DEFAULT_CONFIG_PATH,
        hide_default_value = true,
        help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"
    )]
    config: String,
    #[arg(long, help = "Output machine-readable JSON")]
    json: bool,
    #[arg(long, help = "List removals without deleting files")]
    dry_run: bool,
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
