# Config Lifecycle Tutorial

This tutorial covers all config mutation commands:

- `add`
- `remove`
- `set`
- `list`
- `config export`
- `config import`

## Audience

- Users managing multiple skills over time
- Teams wanting CLI-first config changes (instead of manual TOML edits)

## Config as Code

`skills.toml` is the single source of truth for your skill state. It is
auto-created on first `install`, but you can also author it manually for full
control:

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
`skills.lock` pins resolved commit SHAs and target paths so that every
`apply` on any machine produces the same result.

## Setup

```bash
eden-skills init --config ./skills.lifecycle.toml
```

## 1) Add a Skill

Example: add a copy-mode skill for Cursor.

```bash
eden-skills add \
  --config ./skills.lifecycle.toml \
  --id search-tool \
  --repo https://github.com/vercel-labs/skills.git \
  --subpath packages/search \
  --ref main \
  --mode copy \
  --target cursor \
  --verify-enabled true \
  --verify-check path-exists content-present \
  --no-exec-metadata-only false
```

Target spec rules for `add`/`set`:

- `claude-code`
- `cursor`
- `custom:<path>`

`add` fails if `--id` already exists.

## 2) Inspect Current Inventory

Text:

```bash
eden-skills list --config ./skills.lifecycle.toml
```

JSON:

```bash
eden-skills list --config ./skills.lifecycle.toml --json
```

The JSON payload includes `count` and per-skill fields (`id`, `source`, `install`, `verify`, `targets`).

## 3) Update an Existing Skill

Example: switch mode, checks, and target list.

```bash
eden-skills set \
  --config ./skills.lifecycle.toml \
  search-tool \
  --mode symlink \
  --verify-check path-exists target-resolves is-symlink \
  --target claude-code cursor
```

Notes:

- At least one mutation flag is required.
- `set` only mutates fields you explicitly pass.

## 4) Remove a Skill

```bash
eden-skills remove --config ./skills.lifecycle.toml search-tool
```

Behavior:

- Removes only the matching skill entry.
- Also runs uninstall cleanup on installed target paths via adapter logic.
- Updates co-located lock state to keep `skills.lifecycle.lock` aligned.

Batch remove (multiple IDs):

```bash
eden-skills remove --config ./skills.lifecycle.toml skill-a skill-b
```

Atomic validation applies to batch mode: if any ID is unknown, no removal is performed.

Interactive remove (TTY only, no args):

```bash
eden-skills remove --config ./skills.lifecycle.toml
```

This opens a checkbox selector so you can toggle multiple skills before the
final confirmation prompt. The old `*` wildcard shortcut is no longer special;
use the selector or pass explicit IDs.

Non-interactive confirmation skip:

```bash
eden-skills remove --config ./skills.lifecycle.toml skill-a -y
```

Remove and clean orphaned repo-cache entries afterwards:

```bash
eden-skills remove --config ./skills.lifecycle.toml skill-a --auto-clean
```

JSON output for automation:

```bash
eden-skills remove --config ./skills.lifecycle.toml skill-a skill-b --json
```

The JSON payload includes a `removed` array. When `--auto-clean` is used, it
also adds a nested `clean` object with removed cache paths and freed bytes.

Docker ownership note:

- If a target directory is marked as externally managed in `.eden-managed`,
  `remove` defaults to config-only removal and keeps the files in place.
- Use `--force` when you intentionally want to delete those files and clear the
  manifest entry too.

## 5) Export a Normalized Config

Plain TOML:

```bash
eden-skills config export --config ./skills.lifecycle.toml
```

JSON wrapper:

```bash
eden-skills config export --config ./skills.lifecycle.toml --json
```

## 6) Import a Config

Preview (no write):

```bash
eden-skills config import \
  --from ./skills.lifecycle.toml \
  --config ./skills.imported.toml \
  --dry-run
```

Write to destination:

```bash
eden-skills config import \
  --from ./skills.lifecycle.toml \
  --config ./skills.imported.toml
```

## Recommended Validation Loop

After any mutation:

```bash
eden-skills plan --config ./skills.lifecycle.toml
eden-skills doctor --config ./skills.lifecycle.toml
eden-skills clean --config ./skills.lifecycle.toml
```

If drift is reported:

```bash
eden-skills repair --config ./skills.lifecycle.toml
```
