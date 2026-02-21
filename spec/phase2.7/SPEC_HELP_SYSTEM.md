# SPEC_HELP_SYSTEM.md

Help text, version information, and command grouping for `eden-skills`.

**Related contracts:**

- `spec/phase1/SPEC_COMMANDS.md` (base command surface)
- `spec/phase2/SPEC_COMMANDS_EXT.md` (Phase 2 extensions)
- `spec/phase2.5/SPEC_INSTALL_URL.md` (URL-mode install)

## 1. Purpose

The current CLI produces help output with no descriptions for commands
or arguments. This spec defines the complete help text contract so that
`eden-skills --help` and every subcommand's `--help` are self-documenting
and production-quality.

## 2. Version Information

### 2.1 `--version` Flag

The root CLI MUST support `--version` (and its short form `-V`):

```text
$ eden-skills --version
eden-skills 0.1.0
```

The version string MUST be sourced from `Cargo.toml` via `clap`'s
built-in `#[command(version)]` attribute.

### 2.2 Version in Help Header

The root `--help` output MUST include the version in the header:

```text
eden-skills 0.1.0
Deterministic skill installation and reconciliation for agent environments.

Usage: eden-skills [OPTIONS] <COMMAND>
```

## 3. Root Help Text

### 3.1 About Text

The root command MUST have both `about` (one-line) and `long_about`
(expanded) descriptions:

- **about:** `"Deterministic skill installation and reconciliation for agent environments"`
- **long_about:** A 2–3 sentence expansion explaining that eden-skills
  manages the full lifecycle of agent skills via configuration-driven
  plan/apply/verify/repair.

### 3.2 Command Grouping

Commands MUST be organized into logical groups using clap `help_heading`.
The following groups and ordering are normative:

```text
Install & Update:
  install   Install skills from a URL, path, or registry
  update    Refresh registry sources
  remove    Uninstall a skill and clean up its files

State Reconciliation:
  plan      Preview planned actions without making changes
  apply     Reconcile installed state with configuration
  doctor    Diagnose configuration and installation health
  repair    Auto-repair drifted or broken installations

Configuration:
  init      Create a new skills.toml configuration file
  list      List configured skills and their targets
  add       Add a skill entry to skills.toml
  set       Modify properties of an existing skill entry
  config    Export or import configuration
```

### 3.3 After-Help Examples

The root `--help` MUST include an `after_help` section with quickstart
examples:

```text
Examples:
  eden-skills install vercel-labs/agent-skills    Install skills from GitHub
  eden-skills install ./my-local-skill            Install from local path
  eden-skills list                                Show configured skills
  eden-skills doctor                              Check installation health

Documentation: https://github.com/AI-Eden/eden-skills
```

## 4. Subcommand Help Text

Every subcommand MUST have an `about` description. The following table
defines the normative `about` text for each command:

| Command | `about` Text |
| :--- | :--- |
| `install` | `"Install skills from a URL, path, or registry"` |
| `update` | `"Refresh registry sources to latest versions"` |
| `remove` | `"Uninstall a skill and clean up its files"` |
| `plan` | `"Preview planned actions without making changes"` |
| `apply` | `"Reconcile installed state with configuration"` |
| `doctor` | `"Diagnose configuration and installation health"` |
| `repair` | `"Auto-repair drifted or broken installations"` |
| `init` | `"Create a new skills.toml configuration file"` |
| `list` | `"List configured skills and their targets"` |
| `add` | `"Add a skill entry to skills.toml"` |
| `set` | `"Modify properties of an existing skill entry"` |
| `config` | `"Export or import configuration"` |
| `config export` | `"Export configuration to stdout"` |
| `config import` | `"Import configuration from another file"` |

## 5. Argument Help Text

Every argument and option MUST have a `help` annotation. The following
tables define the normative help text.

### 5.1 Global Options

| Option | Help Text |
| :--- | :--- |
| `--config <PATH>` | `"Path to skills.toml config file [default: ~/.eden-skills/skills.toml]"` |
| `--strict` | `"Exit with error on drift or warnings"` |
| `--json` | `"Output machine-readable JSON"` |
| `--color <WHEN>` | `"Control color output: auto, always, never [default: auto]"` |
| `--concurrency <N>` | `"Maximum number of concurrent operations"` |

### 5.2 `install` Arguments

| Argument / Option | Help Text |
| :--- | :--- |
| `<SOURCE>` | `"URL, local path, or registry skill name"` |
| `--id <ID>` | `"Override the auto-derived skill identifier"` |
| `--ref <REF>` | `"Git reference (branch, tag, or commit)"` |
| `-s, --skill <NAME>...` | `"Install only the named skill(s) from the repository"` |
| `--all` | `"Install all discovered skills without confirmation"` |
| `-y, --yes` | `"Skip all interactive confirmation prompts"` |
| `--list` | `"List discovered skills without installing"` |
| `--version <CONSTRAINT>` | `"Version constraint for registry mode (e.g. >=1.0)"` |
| `--registry <NAME>` | `"Use a specific registry for resolution"` |
| `-t, --target <SPEC>...` | `"Install to specific agent targets (e.g. claude-code, cursor)"` |
| `--dry-run` | `"Preview what would be installed without making changes"` |
| `--copy` | `"Use file copy instead of symlinks"` |

### 5.3 `remove` Arguments

| Argument / Option | Help Text |
| :--- | :--- |
| `<SKILL_ID>...` | `"One or more skill identifiers to remove"` |
| `-y, --yes` | `"Skip confirmation prompt"` |

### 5.4 `add` Arguments

| Argument / Option | Help Text |
| :--- | :--- |
| `--id <ID>` | `"Unique skill identifier"` |
| `--repo <URL>` | `"Source repository URL"` |
| `--ref <REF>` | `"Git reference [default: main]"` |
| `--subpath <PATH>` | `"Subdirectory within the repository [default: .]"` |
| `--mode <MODE>` | `"Install mode: symlink or copy [default: symlink]"` |
| `-t, --target <SPEC>...` | `"Agent targets (e.g. claude-code, cursor, custom:/path)"` |
| `--verify-enabled <BOOL>` | `"Enable post-install verification"` |
| `--verify-check <CHECK>...` | `"Verification checks to run"` |
| `--no-exec-metadata-only <BOOL>` | `"Metadata-only mode (skip file installation)"` |

### 5.5 `set` Arguments

| Argument / Option | Help Text |
| :--- | :--- |
| `<SKILL_ID>` | `"Skill identifier to modify"` |
| `--repo <URL>` | `"New source repository URL"` |
| `--ref <REF>` | `"New Git reference"` |
| `--subpath <PATH>` | `"New subdirectory within the repository"` |
| `--mode <MODE>` | `"New install mode: symlink or copy"` |
| `-t, --target <SPEC>...` | `"Replace all targets"` |
| `--verify-enabled <BOOL>` | `"Enable or disable verification"` |
| `--verify-check <CHECK>...` | `"Replace verification checks"` |
| `--no-exec-metadata-only <BOOL>` | `"Set metadata-only mode"` |

### 5.6 `init` Arguments

| Argument / Option | Help Text |
| :--- | :--- |
| `--force` | `"Overwrite existing config file"` |

### 5.7 `config import` Arguments

| Argument / Option | Help Text |
| :--- | :--- |
| `--from <PATH>` | `"Path to the source config file to import"` |
| `--dry-run` | `"Preview import without writing changes"` |

## 6. Short Flags

The following short flags MUST be added to improve ergonomics:

| Long Flag | Short Flag | Commands |
| :--- | :--- | :--- |
| `--skill` | `-s` | `install` |
| `--target` | `-t` | `install`, `add`, `set`, `remove` (Phase 2.7 enhanced) |
| `--yes` | `-y` | `install`, `remove` |
| `--version` (root) | `-V` | root |

Short flags MUST NOT conflict with each other within the same command
scope. The `-h` short flag is reserved for `--help` by clap.

## 7. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **HLP-001** | Builder | **P0** | Root CLI MUST support `--version` / `-V` showing package version. | `eden-skills --version` outputs `eden-skills <version>`. |
| **HLP-002** | Builder | **P0** | Root `--help` MUST show version, about text, grouped commands, and examples. | `eden-skills --help` contains all Section 3 elements. |
| **HLP-003** | Builder | **P0** | Every subcommand MUST have an `about` description per Section 4 table. | Each `eden-skills <cmd> --help` shows the documented about text. |
| **HLP-004** | Builder | **P0** | Every argument and option MUST have a `help` annotation per Section 5. | No `--help` output contains blank description fields. |
| **HLP-005** | Builder | **P0** | Commands MUST be grouped with headings per Section 3.2. | `--help` output shows `Install & Update`, `State Reconciliation`, `Configuration` groups. |
| **HLP-006** | Builder | **P1** | Short flags `-s`, `-t`, `-y`, `-V` MUST be available per Section 6. | `eden-skills install -s browser-tool -t cursor -y ...` succeeds. |
| **HLP-007** | Builder | **P1** | `install` MUST accept `--copy` flag to set install mode. | `eden-skills install owner/repo --copy` persists `install.mode = "copy"`. |

## 8. Backward Compatibility

| Existing Feature | Phase 2.7 Behavior |
| :--- | :--- |
| Long flags (`--skill`, `--target`) | Unchanged. Short aliases are additive. |
| `--help` output format | Enhanced with descriptions. Existing integrations parsing `--help` SHOULD NOT break (output is additive). |
| `add --mode symlink` | Unchanged. `install --copy` is a new convenience alias. |
