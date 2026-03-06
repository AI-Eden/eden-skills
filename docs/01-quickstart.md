# Quickstart: First Successful Run

This guide gets you from a fresh install to working skills in under two minutes.

## Prerequisites

- Git
- `eden-skills` installed

Linux / macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/AI-Eden/eden-skills/main/install.sh | bash
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/AI-Eden/eden-skills/main/install.ps1 | iex
```

Alternative:

```bash
cargo install eden-skills --locked
```

---

## Path 1: Install from a URL (fastest)

The simplest way to get started. No config file needed.

```bash
# Install all skills from a GitHub repo
eden-skills install vercel-labs/agent-skills --all

# Or pick skills interactively
eden-skills install vercel-labs/agent-skills
```

`eden-skills` auto-detects which agents you have installed (Claude Code, Cursor, Codex, Windsurf) and links each skill to the correct directory.

Verify:

```bash
eden-skills list
eden-skills doctor
```

Expected `doctor` output when everything is healthy:

```
Doctor   ✓ no issues detected
```

If `doctor` reports a broken link, run:

```bash
eden-skills repair
```

That is it. Skills are installed and ready.

---

## Path 2: Config-Driven Setup (recommended for teams)

Use `skills.toml` for repeatable, version-controlled skill state. Every `apply` on any machine produces the same result.

### Step 1: Initialize a config

```bash
eden-skills init
```

Creates `~/.eden-skills/skills.toml` and a co-located `skills.lock`.

To use a project-local config instead:

```bash
eden-skills init --config ./skills.toml
```

### Step 2: Add a skill

Install from URL — eden-skills writes the config entry automatically:

```bash
eden-skills install vercel-labs/agent-skills --skill frontend-design
```

Or add an entry manually:

```bash
eden-skills add \
  --id frontend-design \
  --repo https://github.com/vercel-labs/agent-skills.git \
  --subpath skills/frontend-design \
  --ref main \
  --target claude-code
```

### Step 3: Preview planned changes

```bash
eden-skills plan
```

`plan` is read-only. Typical action types:

- `create` — target does not exist yet
- `noop` — already in sync
- `conflict` — local state disagrees with config
- `remove` — lock-only orphan to be cleaned up

Machine-readable:

```bash
eden-skills plan --json
```

### Step 4: Apply

```bash
eden-skills apply
```

Expected output:

```
Syncing   1 cloned, 0 updated, 0 skipped, 0 failed
Safety    1 permissive, 0 non-permissive, 0 unknown
Install   ✓ frontend-design → ~/.claude/skills/frontend-design (symlink)
Summary   ✓ 1 created, 0 updated, 0 noop, 0 conflicts
✓ Verification passed
```

`apply` also writes `skills.lock` to track installed commit SHAs and target paths.

### Step 5: Diagnose

```bash
eden-skills doctor
```

Machine-readable diagnostics:

```bash
eden-skills doctor --json
```

### Step 6: Repair drift (when needed)

If you manually delete a symlink or move source files:

```bash
eden-skills repair
```

`repair` reuses the same planning and verification contracts as `apply` and self-heals without manual relinking.

---

## Useful Flags

```bash
eden-skills --version                        # Check version
eden-skills --help                           # Global help
eden-skills install --help                   # Command-specific help
eden-skills apply --json                     # Machine-readable output
eden-skills apply --color never              # Disable color output
eden-skills apply --concurrency 4            # Limit parallel tasks
```

---

## What to Learn Next

- Manage multi-skill configs (add / remove / set / list): [02-config-lifecycle.md](02-config-lifecycle.md)
- Install from registries by name and version: [03-registry-and-install.md](03-registry-and-install.md)
- Docker targets: [04-docker-targets.md](04-docker-targets.md)
