# Registry and Install Tutorial (Phase 2)

This tutorial explains the Phase 2 registry workflow and the `install` command.

## Audience

- Users who want to install skills by name/version instead of direct Git entries
- Teams maintaining shared internal/public registries

## Concepts

- **Mode A skill**: direct Git source (`id` + `source`)
- **Mode B skill**: registry-resolved (`name` + `version` + optional `registry`)

`install` writes Mode B entries to `skills.toml`, then resolves them to concrete Git sources.

## Prerequisites

- A valid config file
- At least one registry under `[registries]`
- Registry cache initialized via `update`

Recommended setup (if config does not exist yet):

```bash
CONFIG="${HOME}/.eden-skills/skills.toml"
eden-skills init --config "$CONFIG"
```

## 1) Configure Registries

Add a section like:

```toml
[registries]
official = { url = "https://github.com/eden-skills/registry-official.git", priority = 100 }
forge = { url = "https://github.com/eden-skills/registry-forge.git", priority = 10 }
```

Priority rule:

- Higher `priority` is searched first.

## 2) Sync Registry Indexes

```bash
eden-skills update --config "$CONFIG"
```

Useful options:

- `--concurrency <n>`
- `--json`

Important behavior:

- Uses shallow clone/fetch (`--depth 1`)
- Allows partial failures (command can still return success with warnings)

## 3) Install by Skill Name

Default target (local):

```bash
eden-skills install browser-tool --config "$CONFIG"
```

Pin version or range:

```bash
eden-skills install browser-tool --config "$CONFIG" --version "^2.0"
```

Restrict to one registry:

```bash
eden-skills install browser-tool --config "$CONFIG" --registry official
```

Override target for this install:

```bash
eden-skills install browser-tool --config "$CONFIG" --target docker:my-agent
```

Target format for `install --target`:

- `local`
- `docker:<container>`

`install --target` updates the skill target definition in config and runs the standard install pipeline.  
For Docker-target operational checks and caveats, continue with `04-docker-targets.md`.

Install mode override for this request:

```bash
eden-skills install browser-tool --config "$CONFIG" --copy
```

This persists `install.mode = "copy"` for the installed skill entry.

Skip confirmation prompts in interactive install paths:

```bash
eden-skills install browser-tool --config "$CONFIG" -y
```

## 4) Dry-Run Before Writing

```bash
eden-skills install browser-tool --config "$CONFIG" --version "~2.3" --dry-run
```

Dry-run behavior:

- Shows resolved source + target info
- Does not mutate config
- Does not install files

## 5) Apply Full Config Reconciliation

Even after `install`, running a full reconciliation is recommended:

```bash
eden-skills apply --config "$CONFIG"
```

This ensures all skills (Mode A + Mode B) converge to desired state.
Mutating install paths also refresh lock state (`skills.lock`) for reconciliation.

## Common Errors

- `Registry index not found. Run eden-skills update first.`  
  Run `update` before `install`.

- `UNKNOWN_REGISTRY`  
  `skills.toml` references a registry name not defined under `[registries]`.

- `INVALID_SEMVER`  
  Version constraint string is invalid.

## Version Selection Notes

When resolving versions:

- Yanked versions are excluded.
- If no explicit constraint is given, highest stable release is preferred.
- Exact version pins are honored when available.
