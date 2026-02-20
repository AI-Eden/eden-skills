# SPEC_SCHEMA.md

Normative schema for `~/.eden-skills/skills.toml`.

## 1. Top-Level Structure

Config file MUST be valid TOML and follow:

```toml
version = 1

[storage]
root = "~/.local/share/eden-skills/repos"

[[skills]]
id = "browser-tool"

[skills.source]
repo = "https://github.com/vercel-labs/skills.git"
subpath = "packages/browser"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "claude-code"

[[skills.targets]]
agent = "cursor"

[[skills.targets]]
agent = "custom"
path = "/opt/my-agent/tools"

[skills.verify]
enabled = true
checks = ["path-exists", "target-resolves", "is-symlink"]

[skills.safety]
no_exec_metadata_only = false
```

## 2. Required Fields

- `version` MUST be `1`
- `skills` MUST exist and contain at least one item
- For each skill item, `id` MUST be unique in file
- For each skill item, `source.repo` MUST be a valid git URL (https, ssh, or file for local/offline workflows)
- For each skill item, `targets` MUST contain at least one target
- For `agent: custom`, `path` MUST be provided

## 3. Optional Fields and Defaults

- `storage.root`: default `~/.local/share/eden-skills/repos`
- `source.ref`: default `main`
- `source.subpath`: default `.`
- `install.mode`: default `symlink` (`symlink|copy`)
- `verify.enabled`: default `true`
- `verify.checks` default for `symlink`: `["path-exists", "target-resolves", "is-symlink"]`
- `verify.checks` default for `copy`: `["path-exists", "content-present"]`
- `safety.no_exec_metadata_only`: default `false`

## 4. Validation Rules

- Unknown top-level keys SHOULD fail in strict mode and warn in default mode.
- Path strings MUST support `~` expansion to user home.
- Relative paths in config MUST resolve relative to config file directory.
- Duplicate skill `id` MUST fail validation.
- Empty `verify.checks` with `verify.enabled=true` MUST fail validation.
- `safety.no_exec_metadata_only=true` MUST be honored by command execution semantics (`apply`/`repair` skip target mutation and verification for that skill while keeping metadata workflows enabled).

## 5. Error Contract

On schema validation failure:

- CLI MUST print machine-readable error code and human-readable message.
- CLI MUST return exit code `2`.
- Error message MUST include field path (example: `skills[1].targets[0].path`).
