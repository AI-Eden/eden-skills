# CLI Reference

Complete reference for all `eden-skills` commands, options, and configuration.

---

## install

Install skills from a GitHub repository, URL, or local path.

```bash
eden-skills install <source> [options]
```

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

When a repository exposes multiple skills and you do not pass `--all` or
`--skill`, `eden-skills` opens an interactive checkbox selector in TTY
sessions. In non-interactive contexts, it falls back to installing all
discovered skills.

---

## remove

Remove installed skills by name, or select interactively.

```bash
eden-skills remove [skills...] [options]
```

Running `eden-skills remove` without arguments in a TTY opens an interactive
checkbox selector, then asks for confirmation before deleting.

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

# Remove and clean orphaned repo-cache entries afterwards
eden-skills remove web-design-guidelines --auto-clean

# Force-delete files for an externally-managed target
eden-skills remove web-design-guidelines --force
```

---

## list

List all installed skills.

```bash
eden-skills list [options]
```

Add `--json` for machine-readable output.

---

## apply

Reconcile all skills to the desired config state. Reads `skills.toml`, clones
or updates sources, and installs to all configured targets. Idempotent.

```bash
eden-skills apply [options]
```

---

## plan

Preview planned changes without writing anything. Same logic as `apply`, but
read-only.

```bash
eden-skills plan [options]
```

---

## doctor

Detect broken symlinks, missing sources, drift, and risk findings.

```bash
eden-skills doctor [options]
```

---

## repair

Self-heal broken symlinks and drifted state. Uses the same planning and
verification logic as `apply`.

```bash
eden-skills repair [options]
```

---

## update

Sync registry indexes to latest.

```bash
eden-skills update [options]
```

---

## clean

Remove orphaned repo-cache entries and stale discovery temp directories.

```bash
eden-skills clean [options]
```

---

## init

Initialize a new `skills.toml` config file.

```bash
eden-skills init [options]
```

---

## add / set

Add a new skill entry or update an existing entry in config.

```bash
eden-skills add --id <id> --repo <url> [--subpath <path>] [--ref <ref>] [--target <agent>] [options]
eden-skills set <id> [--mode <mode>] [--target <agents...>] [options]
```

---

## config export / config import

Export a normalized config or import and validate a config from another file.

```bash
eden-skills config export [options]
eden-skills config import --from <path> [options]
```

---

## docker mount-hint

Show recommended bind mount configuration for a Docker container.

```bash
eden-skills docker mount-hint <container>
```

---

## Config as Code

`~/.eden-skills/skills.toml` is auto-created on first `install`. You can also
author it manually for full control over your skill state:

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

Run `eden-skills apply` to converge the system to this config. The co-located
`skills.lock` tracks resolved commit SHAs and target paths for deterministic
reproducibility.

---

## Global Options

These flags are available on all commands:

| Option | Description |
| --- | --- |
| `--config <path>` | Config file path (default: `~/.eden-skills/skills.toml`) |
| `--strict` | Treat drift and warnings as hard failures |
| `--json` | Machine-readable output |
| `--color <auto\|always\|never>` | ANSI color policy |
| `--concurrency <n>` | Parallel task limit for `apply`, `repair`, `update` |
| `--version` / `-V` | Print CLI version |

---

## Exit Codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | Runtime failure |
| `2` | Config / validation failure |
| `3` | Strict-mode conflict or drift |

---

## Supported Agents

Agent directories are auto-detected on `install`. Override with `--target`.
Shared-path aliases are supported: `amp`, `kimi-cli`, `replit`, and `universal`
all map to `~/.config/agents/skills`.

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
