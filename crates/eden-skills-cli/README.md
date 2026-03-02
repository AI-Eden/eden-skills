# eden-skills

Deterministic skill manager for AI agent environments (Claude Code, Cursor, Codex, Windsurf, and more).

For full documentation, see the [GitHub repository](https://github.com/AI-Eden/eden-skills).

## Install

```bash
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
```

## License

MIT
