# eden-skills

Deterministic skill manager for AI agent environments (Claude Code, Cursor, Codex, Windsurf, and 40+ more).

For full documentation, see the [GitHub repository](https://github.com/AI-Eden/eden-skills).

## Install

```bash
# One-line install (Linux / macOS)
curl -fsSL https://raw.githubusercontent.com/AI-Eden/eden-skills/main/install.sh | bash

# Or via Cargo
cargo install eden-skills --locked
```

## Quick Start

```bash
# Install skills from a GitHub repo (auto-detects your agents)
eden-skills install vercel-labs/agent-skills --all

# List installed skills
eden-skills list

# Check for drift or broken installs
eden-skills doctor

# Self-heal broken symlinks
eden-skills repair
```

## Commands

| Command | Description |
|---------|-------------|
| `install <source>` | Install skills from URL, local path, or registry |
| `remove [ids...]` | Remove skills (interactive multi-select when no args) |
| `update` | Refresh sources and re-apply all skills |
| `list` | Show installed skills in a styled table |
| `apply` | Apply config to all targets |
| `doctor` | Diagnose config, symlink, and Docker issues |
| `repair` | Auto-fix issues found by `doctor` |
| `clean` | Remove orphaned cache entries |
| `plan` | Preview what `apply` would do |
| `init` / `add` / `set` | Manage `skills.toml` configuration |
| `config export/import` | Portable config sharing |
| `docker mount-hint` | Print recommended bind-mount flags |

## License

MIT
