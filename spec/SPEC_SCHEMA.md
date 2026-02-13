# SPEC_SCHEMA.md

Normative schema for `~/.config/eden-skills/skills.yaml`.

## 1. Top-Level Structure

Config file MUST be valid YAML and follow:

```yaml
version: 1
storage:
  root: "~/.local/share/eden-skills/repos"
skills:
  - id: "browser-tool"
    source:
      repo: "https://github.com/vercel-labs/skills.git"
      subpath: "packages/browser"
      ref: "main"
    install:
      mode: "symlink"
    targets:
      - agent: "claude-code"
      - agent: "cursor"
      - agent: "custom"
        path: "/opt/my-agent/tools"
    verify:
      enabled: true
      checks: ["path-exists", "target-resolves", "is-symlink"]
    safety:
      no_exec_metadata_only: false
```

## 2. Required Fields

- `version` MUST be `1`
- `skills` MUST exist and contain at least one item
- For each skill item, `id` MUST be unique in file
- For each skill item, `source.repo` MUST be a valid git URL (https or ssh)
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

## 5. Error Contract

On schema validation failure:

- CLI MUST print machine-readable error code and human-readable message.
- CLI MUST return exit code `2`.
- Error message MUST include field path (example: `skills[1].targets[0].path`).
