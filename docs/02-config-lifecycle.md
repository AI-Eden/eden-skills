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

## Setup

```bash
ES="cargo run -p eden-skills-cli --"
$ES init --config ./skills.lifecycle.toml
```

## 1) Add a Skill

Example: add a copy-mode skill for Cursor.

```bash
$ES add \
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
$ES list --config ./skills.lifecycle.toml
```

JSON:

```bash
$ES list --config ./skills.lifecycle.toml --json
```

The JSON payload includes `count` and per-skill fields (`id`, `source`, `install`, `verify`, `targets`).

## 3) Update an Existing Skill

Example: switch mode, checks, and target list.

```bash
$ES set \
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
$ES remove --config ./skills.lifecycle.toml search-tool
```

Behavior:

- Removes only the matching skill entry.
- Also runs uninstall cleanup on installed target paths via adapter logic.

## 5) Export a Normalized Config

Plain TOML:

```bash
$ES config export --config ./skills.lifecycle.toml
```

JSON wrapper:

```bash
$ES config export --config ./skills.lifecycle.toml --json
```

## 6) Import a Config

Preview (no write):

```bash
$ES config import \
  --from ./skills.lifecycle.toml \
  --config ./skills.imported.toml \
  --dry-run
```

Write to destination:

```bash
$ES config import \
  --from ./skills.lifecycle.toml \
  --config ./skills.imported.toml
```

## Recommended Validation Loop

After any mutation:

```bash
$ES plan --config ./skills.lifecycle.toml
$ES doctor --config ./skills.lifecycle.toml
```

If drift is reported:

```bash
$ES repair --config ./skills.lifecycle.toml
```
