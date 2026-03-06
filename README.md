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
`$env:USERPROFILE\.eden-skills\bin\` on Windows and print PATH guidance when needed.

**Alternative: cargo install**

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

### Options

| Option | Description |
| --- | --- |
| `-s, --skill <name>` | Install a specific skill by name |
| `--all` | Install all discovered skills without prompts |
| `-t, --target <agent>` | Override target agent (`local`, `docker:<container>`) |
| `--copy` | Copy files instead of symlinking |
| `-y, --yes` | Skip confirmation prompts |
| `--list` | List available skills without installing |
| `--dry-run` | Preview changes without writing anything |

### Examples

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

## Other Commands

| Command | Description |
| --- | --- |
| `eden-skills list` | List installed skills |
| `eden-skills remove [skills...]` | Remove skills (batch or interactive) |
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

Agent directories are auto-detected on `install`. Override with `--target`:

| Agent | `--target` alias | Global Path |
| --- | --- | --- |
| Claude Code | `claude-code` | `~/.claude/skills/` |
| Cursor | `cursor` | `~/.cursor/skills/` |
| Codex | `codex` | `~/.codex/skills/` |
| Windsurf | `windsurf` | `~/.codeium/windsurf/skills/` |
| Docker container | `docker:<name>` | (inside container) |
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
