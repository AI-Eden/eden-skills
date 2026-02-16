# SPEC_COMMANDS_EXT.md

Phase 2 command extensions for the `eden-skills` CLI.

**Base contract:** `spec/phase1/SPEC_COMMANDS.md`
**Rule:** This file defines additive commands and flag extensions only.
It MUST NOT contradict or override Phase 1 command semantics.

## 1. Purpose

Define new CLI commands and flag extensions introduced in Phase 2 for
registry management and environment-targeted installation.

## 2. New Commands

### 2.1 `update`

Purpose: synchronize local registry indexes with their remote Git sources.

CLI shape:

- `eden-skills update [--config <path>]`

Behavior:

- MUST read `[registries]` from config.
- MUST clone or pull each registry index repo into `~/.eden/registries/<name>/`.
- MUST execute registry syncs concurrently (via Reactor, bounded by ARC-002).
- MUST report per-registry sync status (`cloned`, `updated`, `skipped`, `failed`).
- MUST NOT fail the entire command if one registry fails (partial success allowed).
- When `[registries]` is absent or empty, MUST emit a warning and exit code `0`.

Output:

```text
registry sync: official=updated forge=skipped (0 failed)
```

JSON output (`--json`):

```json
{
  "registries": [
    { "name": "official", "status": "updated", "url": "..." },
    { "name": "forge", "status": "skipped", "url": "..." }
  ],
  "failed": 0
}
```

### 2.2 `install` (Registry Mode)

Purpose: install a skill by name using registry resolution.

CLI shape:

- `eden-skills install <skill-name> [--version <constraint>] [--registry <name>] [--target <target-spec>] [--config <path>]`

Behavior:

- MUST resolve `<skill-name>` using the registry resolution logic (ARC-202).
- MUST respect `--version` constraint for SemVer matching.
- MUST respect `--registry` constraint to limit search scope.
- MUST clone/checkout the resolved skill source.
- MUST install to the specified `--target` (or default local targets).
- If the skill is already in `skills.toml`, MUST update the existing entry.
- If the skill is not in `skills.toml`, MUST append a new entry.
- MUST validate and persist the updated config.

Target spec format:

- `local` (default): install via `LocalAdapter`
- `docker:<container>`: install via `DockerAdapter`

### 2.3 Existing Command Extensions

#### `apply` and `repair`

- When config contains Mode B skills (registry-resolved), `apply` and `repair`
  MUST resolve them through the registry before source sync.
- Registry resolution failures MUST be treated as source sync failures
  (same failure semantics as Phase 1 `SPEC_COMMANDS.md` 3.2.1).

#### `doctor`

- MUST emit additional finding codes for Phase 2:
  - `REGISTRY_STALE`: registry index not updated recently.
  - `REGISTRY_UNREACHABLE`: configured registry URL is not accessible.
  - `ADAPTER_HEALTH_FAIL`: target environment health check failed.

## 3. Exit Codes

Phase 2 commands MUST follow the same exit code contract as Phase 1
(`spec/phase1/SPEC_COMMANDS.md` Section 5):

- `0`: success (including partial success for `update`)
- `1`: runtime failure
- `2`: config/schema validation error
- `3`: drift/conflict detected in strict mode

## 4. Global Flag Extensions

No new global flags in Phase 2. Existing `--config`, `--json`, `--strict`
apply to all new commands.
