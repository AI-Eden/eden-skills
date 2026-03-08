# eden-skills

`Deterministic` + `Fast` + `Cross-platform` skill manager for AI agent environments. Auto-detects installed agents (Claude Code, Cursor, Codex and more). Supports Docker containers.  

100% usable on: Linux, macOS, and Windows.

## Install

**Prerequisite:** Git

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/AI-Eden/eden-skills/main/install.sh | bash
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/AI-Eden/eden-skills/main/install.ps1 | iex
```

The install scripts place the binary in `~/.eden-skills/bin/` on Linux/macOS or
`$env:USERPROFILE\.eden-skills\bin\` on Windows. When PATH updates are needed, `install.sh`
updates the selected shell rc file automatically and `install.ps1` updates the user Path.

### Cargo Install Alternative

```bash
cargo install eden-skills --locked
```

Verify:

```bash
eden-skills --version
```

<details> <!-- markdownlint-disable-line -->
<summary>Install from source (development)</summary> <!-- markdownlint-disable-line -->

```bash
git clone https://github.com/AI-Eden/eden-skills.git
cd eden-skills
cargo install --path crates/eden-skills-cli --locked --force
```

</details>

## Install a Skill

```bash
eden-skills install vercel-labs/agent-skills
```

Auto-detects which agents you have installed and links the skill to each.

When a repository exposes multiple skills and you do not pass `--all` or
`--skill`, `eden-skills` opens an interactive checkbox selector in TTY
sessions. In non-interactive contexts, it falls back to installing all
discovered skills.

### Source Formats

```bash
# GitHub shorthand (owner/repo)
eden-skills install vercel-labs/agent-skills

# Full GitHub URL
eden-skills install https://github.com/vercel-labs/agent-skills

# Direct path to a specific skill in a repo
eden-skills install https://github.com/vercel-labs/agent-skills/tree/main/skills/web-design-guidelines

# Local path
eden-skills install ./my-local-skill
```

### Install Options

| Option | Description |
| --- | --- |
| `-s, --skill <name>` | Install a specific skill by name |
| `--all` | Install all discovered skills without prompts |
| `-t, --target <agent>` | Override target selection (`claude-code`, `cursor`, `local`, `docker:<container>`, `custom:<path>`, and other built-in aliases) |
| `--copy` | Copy files instead of symlinking |
| `--force` | Overwrite externally-managed targets and take over ownership |
| `-y, --yes` | Skip confirmation prompts |
| `--list` | List available skills without installing |
| `--dry-run` | Preview changes without writing anything |

### Install Examples

```bash
# List available skills in a repository
eden-skills install vercel-labs/agent-skills --list

# Install a specific skill
eden-skills install vercel-labs/agent-skills --skill web-design-guidelines

# Install all skills without prompts
eden-skills install vercel-labs/agent-skills --all -y

# Install into a running Docker container
eden-skills install vercel-labs/agent-skills --target docker:my-agent

# Preview what would happen
eden-skills install vercel-labs/agent-skills --dry-run
```

## Remove a Skill

```bash
eden-skills remove web-design-guidelines
```

In TTY sessions, running `eden-skills remove` without skill IDs opens the same
checkbox selector used by `install`, then asks for confirmation before deleting
anything.

### Remove Options

| Option | Description |
| --- | --- |
| `-y, --yes` | Skip confirmation prompts |
| `--auto-clean` | Run cache cleanup after removal and report freed space |
| `--force` | Remove files for externally-managed targets instead of config-only removal |
| `--json` | Emit machine-readable remove output (with additive `clean` details when used with `--auto-clean`) |

### Remove Examples

```bash
# Remove one skill
eden-skills remove web-design-guidelines

# Select multiple skills interactively
eden-skills remove

# Remove a skill and clean orphaned repo cache entries afterwards
eden-skills remove web-design-guidelines --auto-clean

# Force-delete files for an externally-managed target
eden-skills remove web-design-guidelines --force
```

## Other Commands

| Command | Description |
| --- | --- |
| `eden-skills list` | List installed skills |
| `eden-skills remove [skills...]` | Remove skills (batch or interactive) |
| `eden-skills clean` | Remove orphaned repo-cache entries and stale discovery directories |
| `eden-skills update` | Sync registry indexes to latest |
| `eden-skills apply` | Reconcile all skills to desired config state |
| `eden-skills doctor` | Detect broken links, drift, and risk findings |
| `eden-skills docker mount-hint <container>` | Show recommended bind mounts for Docker live sync |
| `eden-skills repair` | Self-heal broken symlinks and drifted state |
| `eden-skills plan` | Preview planned changes (read-only) |
| `eden-skills init` | Initialize a new `skills.toml` config |
| `eden-skills add` | Add a skill entry to config |
| `eden-skills set` | Update a skill field in config |
| `eden-skills config export` | Export normalized config |
| `eden-skills config import` | Import and validate a config |

## Why eden-skills

**Installs are deterministic.** `skills.lock` tracks every installed skill, commit SHA, and target path. Run `apply` again on any machine and you get exactly the same state.

**Broken installs self-heal.** `doctor` detects broken symlinks, missing sources, and drift. `repair` fixes them automatically — no manual relinking.

**Config is code.** `skills.toml` is your single source of truth. Version it, share it with your team, and `apply` it anywhere.

**Docker-aware.** Install skills directly into running containers with `--target docker:<container>`, auto-detect installed agents inside the container, and use `eden-skills docker mount-hint <container>` to configure bind mounts for live sync.

**Cache stays tidy.** Use `clean` to remove orphaned repo-cache entries and stale
discovery temp directories, or add `--auto-clean` to `remove` so cleanup runs
immediately after uninstalling skills.

## Config as Code

`~/.eden-skills/skills.toml` is auto-created on first `install`. Example of a manually managed config:

```toml
version = 1

[storage]
root = "~/.eden-skills/skills"

[[skills]]
id = "web-design-guidelines"

[skills.source]
repo = "https://github.com/vercel-labs/agent-skills.git"
subpath = "skills/web-design-guidelines"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "claude-code"
```

Run `eden-skills apply` to converge the system to this config.

## Supported Agents

Agent directories are auto-detected on `install`. Override with `--target`.
Shared-path aliases are supported too: `amp`, `kimi-cli`, `replit`, and
`universal` all map to `~/.config/agents/skills`.

| Agent | `--target` alias | Global Path |
| --- | --- | --- |
| Adal | `adal` | `~/.adal/skills/` |
| Amp | `amp` | `~/.config/agents/skills/` |
| Antigravity | `antigravity` | `~/.gemini/antigravity/skills/` |
| Augment | `augment` | `~/.augment/skills/` |
| Claude Code | `claude-code` | `~/.claude/skills/` |
| Cline | `cline` | `~/.agents/skills/` |
| Codebuddy | `codebuddy` | `~/.codebuddy/skills/` |
| Codex | `codex` | `~/.codex/skills/` |
| Command Code | `command-code` | `~/.commandcode/skills/` |
| Continue | `continue` | `~/.continue/skills/` |
| Cortex | `cortex` | `~/.snowflake/cortex/skills/` |
| Crush | `crush` | `~/.config/crush/skills/` |
| Cursor | `cursor` | `~/.cursor/skills/` |
| Droid | `droid` | `~/.factory/skills/` |
| Gemini CLI | `gemini-cli` | `~/.gemini/skills/` |
| GitHub Copilot | `github-copilot` | `~/.copilot/skills/` |
| Goose | `goose` | `~/.config/goose/skills/` |
| Iflow CLI | `iflow-cli` | `~/.iflow/skills/` |
| Junie | `junie` | `~/.junie/skills/` |
| Kilo | `kilo` | `~/.kilocode/skills/` |
| Kimi CLI | `kimi-cli` | `~/.config/agents/skills/` |
| Kiro CLI | `kiro-cli` | `~/.kiro/skills/` |
| Kode | `kode` | `~/.kode/skills/` |
| Mcpjam | `mcpjam` | `~/.mcpjam/skills/` |
| Mistral Vibe | `mistral-vibe` | `~/.vibe/skills/` |
| Mux | `mux` | `~/.mux/skills/` |
| Neovate | `neovate` | `~/.neovate/skills/` |
| Openclaw | `openclaw` | `~/.openclaw/skills/` |
| Opencode | `opencode` | `~/.config/opencode/skills/` |
| Openhands | `openhands` | `~/.openhands/skills/` |
| Pi | `pi` | `~/.pi/agent/skills/` |
| Pochi | `pochi` | `~/.pochi/skills/` |
| Qoder | `qoder` | `~/.qoder/skills/` |
| Qwen Code | `qwen-code` | `~/.qwen/skills/` |
| Replit | `replit` | `~/.config/agents/skills/` |
| Roo | `roo` | `~/.roo/skills/` |
| Trae | `trae` | `~/.trae/skills/` |
| Trae CN | `trae-cn` | `~/.trae-cn/skills/` |
| Universal | `universal` | `~/.config/agents/skills/` |
| Windsurf | `windsurf` | `~/.codeium/windsurf/skills/` |
| Zencoder | `zencoder` | `~/.zencoder/skills/` |
| Docker container | `docker:<name>` | inside container |
| Custom path | `custom:<path>` | any writable path |

## Documentation

- [Quickstart: First Successful Run](docs/01-quickstart.md)
- [Config Lifecycle Management](docs/02-config-lifecycle.md)
- [Registry and Install Workflow](docs/03-registry-and-install.md)
- [Docker Targets Guide](docs/04-docker-targets.md)
- [Safety, Strict Mode, and Exit Codes](docs/05-safety-strict-and-exit-codes.md)
- [Troubleshooting Playbook](docs/06-troubleshooting.md)

## Global Options

| Option | Description |
| --- | --- |
| `--config <path>` | Config file path (default: `~/.eden-skills/skills.toml`) |
| `--strict` | Treat drift and warnings as hard failures |
| `--json` | Machine-readable output |
| `--color <auto\|always\|never>` | ANSI color policy |
| `--concurrency <n>` | Parallel task limit for `apply`, `repair`, `update` |
| `--version` / `-V` | Print CLI version |

## Exit Codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | Runtime failure |
| `2` | Config / validation failure |
| `3` | Strict-mode conflict or drift |

## Current Status

- Phase 1 (CLI foundation): complete
- Phase 2 (async reactor, Docker adapter, registry): complete
- Phase 2.5 (URL install, agent auto-detection, binary distribution): complete
- Phase 2.7 (lock file, UX polish, batch remove): complete
- Phase 2.8 (TUI deep optimization, table rendering, doc comments): complete
- Phase 2.9 (UX polish, update semantics, output consistency): complete
- Phase 2.95 (repo-cache sync, remove-all wildcard, Windows junctions, Docker bind mounts, install scripts): complete
- Phase 2.97 (update reliability, interactive MultiSelect UX, cache clean, Docker ownership safety): complete
- Phase 3 (crawler / taxonomy / curation): not yet implemented

`eden-skills` is under active development. Avoid production use where breaking changes are not tolerable.

## Repository Layout

- [`crates/eden-skills-core`](crates/eden-skills-core): domain logic (config, plan, verify, safety, reactor, adapter, registry)
- [`crates/eden-skills-cli`](crates/eden-skills-cli): user-facing CLI binary (`eden-skills`)
- [`crates/eden-skills-indexer`](crates/eden-skills-indexer): Phase 3 placeholder
- [`spec/`](spec/): normative behavior contracts
- [`docs/`](docs/): user tutorials and guides

## Spec-First Contract

Behavior is defined in [`spec/`](spec/) before code. See [Spec Index](spec/README.md) for the full contract hierarchy.

## Contributing

Contributions welcome — issues, bug reports, docs, tests, and pull requests.  
Align changes with the [spec-first workflow](spec/) and track updates in [`STATUS.yaml`](STATUS.yaml) / [`EXECUTION_TRACKER.md`](EXECUTION_TRACKER.md).  
See [Roadmap](ROADMAP.md) for strategic milestones.
