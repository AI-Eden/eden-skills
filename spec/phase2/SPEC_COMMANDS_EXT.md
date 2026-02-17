# SPEC_COMMANDS_EXT.md

Phase 2 command extensions for the `eden-skills` CLI.

**Base contract:** `spec/phase1/SPEC_COMMANDS.md`
**Rule:** This file defines additive commands and flag extensions only.
It MUST NOT contradict or override Phase 1 command semantics.

## 1. Purpose

Define new CLI commands and flag extensions introduced in Phase 2 for
registry management, environment-targeted installation, and concurrency control.

## 2. New Commands

### 2.1 `update`

Purpose: synchronize local registry indexes with their remote Git sources.

CLI shape:

- `eden-skills update [--config <path>] [--concurrency <n>]`

Behavior:

- MUST read `[registries]` from config.
- MUST clone or pull each registry index repo into `~/.eden-skills/registries/<name>/`.
- MUST use shallow clone (`--depth 1`) for initial clone and shallow fetch for updates (ARC-205).
- MUST execute registry syncs concurrently (via Reactor, bounded by ARC-002).
- MUST report per-registry sync status (`cloned`, `updated`, `skipped`, `failed`).
- MUST report elapsed time for the sync operation.
- MUST NOT fail the entire command if one registry fails (partial success allowed).
- When `[registries]` is absent or empty, MUST emit a warning and exit code `0`.

Output:

```text
registry sync: official=updated forge=skipped (0 failed) [1.2s]
```

JSON output (`--json`):

```json
{
  "registries": [
    { "name": "official", "status": "updated", "url": "..." },
    { "name": "forge", "status": "skipped", "url": "..." }
  ],
  "failed": 0,
  "elapsed_ms": 1200
}
```

### 2.2 `install` (Registry Mode)

Purpose: install a skill by name using registry resolution.

CLI shape:

- `eden-skills install <skill-name> [--version <constraint>] [--registry <name>] [--target <target-spec>] [--config <path>] [--dry-run]`

Behavior:

- MUST resolve `<skill-name>` using the registry resolution logic (ARC-202).
- MUST respect `--version` constraint for SemVer matching (ARC-207).
- MUST respect `--registry` constraint to limit search scope.
- MUST clone/checkout the resolved skill source.
- MUST install to the specified `--target` (or default local targets).
- If the skill is already in `skills.toml`, MUST update the existing entry.
- If the skill is not in `skills.toml`, MUST append a new entry.
- MUST validate and persist the updated config.
- When `--dry-run` is set, MUST display resolved source and target info
  without executing install or modifying config.

Target spec format:

- `local` (default): install via `LocalAdapter`
- `docker:<container>`: install via `DockerAdapter`

### 2.3 Existing Command Extensions

#### `apply` and `repair`

- When config contains Mode B skills (registry-resolved), `apply` and `repair`
  MUST resolve them through the registry before source sync.
- Registry resolution failures MUST be treated as source sync failures
  (same failure semantics as Phase 1 `SPEC_COMMANDS.md` 3.2.1).
- Source sync for all skills (Mode A and resolved Mode B) MUST use the
  Reactor for concurrent execution (ARC-002, ARC-005).

#### `doctor`

- MUST emit additional finding codes for Phase 2:
  - `REGISTRY_STALE`: registry index not updated recently.
  - `REGISTRY_UNREACHABLE`: configured registry URL is not accessible.
  - `ADAPTER_HEALTH_FAIL`: target environment health check failed.
  - `DOCKER_NOT_FOUND`: `docker` CLI not in PATH for Docker targets.

## 3. Exit Codes

Phase 2 commands MUST follow the same exit code contract as Phase 1
(`spec/phase1/SPEC_COMMANDS.md` Section 5):

- `0`: success (including partial success for `update`)
- `1`: runtime failure
- `2`: config/schema validation error
- `3`: drift/conflict detected in strict mode

## 4. Global Flag Extensions

### `--concurrency <n>`

- Applies to: `apply`, `repair`, `update`.
- Overrides `[reactor].concurrency` config value.
- MUST be a positive integer in range `[1, 100]`.
- When not provided, falls back to config value, then built-in default (10).

Priority: `--concurrency` flag > `[reactor].concurrency` config > default (10).

## 5. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **CMD-P2-001** | Builder | **P0** | `update` command syncs registry indexes concurrently. | Multiple registries synced in parallel; elapsed time less than serial. |
| **CMD-P2-002** | Builder | **P0** | `install` command resolves skills from registry by name. | `eden-skills install google-search` produces correct skill installation. |
| **CMD-P2-003** | Builder | **P0** | `apply`/`repair` resolve Mode B skills through registry before source sync. | Mode B skill in config is resolved and installed by `apply`. |
| **CMD-P2-004** | Builder | **P1** | `doctor` emits Phase 2 finding codes (`REGISTRY_STALE`, `ADAPTER_HEALTH_FAIL`, `DOCKER_NOT_FOUND`). | Doctor output includes new finding codes for Phase 2 conditions. |
| **CMD-P2-005** | Builder | **P1** | `--concurrency` global flag overrides reactor concurrency for `apply`, `repair`, `update`. | `--concurrency 1` produces serial behavior; `--concurrency 50` allows 50 parallel tasks. |
| **CMD-P2-006** | Builder | **P1** | `install --dry-run` displays resolution and target info without side effects. | Dry-run produces output but does not modify config or filesystem. |

## 6. Freeze Candidates

| ID | Item | Options Under Consideration | Resolution Needed |
| :--- | :--- | :--- | :--- |
| **FC-C1** | `install` auto-update behavior | Auto-run `update` if index missing vs always require explicit `update` first vs prompt user | Decide if `install` should auto-sync registries. Current recommendation: fail with "run update first" for predictability. |
| **FC-C2** | `update` default scope | Update all registries vs update only registries referenced by current config skills | Decide if `update` syncs all configured registries or only those needed. |
| **FC-C3** | `install` config persistence | Always append to `skills.toml` vs append only with `--save` flag | Decide if `install` auto-persists to config or requires opt-in. Current recommendation: always persist. |
