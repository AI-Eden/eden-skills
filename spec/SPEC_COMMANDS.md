# SPEC_COMMANDS.md

Normative command contract for `eden-skills` CLI.

## 1. Global Flags

- `--config <path>`: config file path, default `~/.config/eden-skills/skills.toml`
- `--json`: emit machine-readable output
- `--strict`: unknown keys and warnings become errors

## 2. Command Model

The CLI is `CLI-first, file-backed`.

- `skills.toml` remains the source of truth.
- Users SHOULD be able to operate via commands without manual file edits.
- Config mutation commands MUST preserve deterministic ordering and idempotent semantics.

## 3. Phase 1 Mandatory Commands

These commands MUST be implemented in Phase 1:

- `plan`
- `apply`
- `doctor`
- `repair`

### 3.1 `plan`

Purpose: compute dry-run action graph without mutating filesystem.

Output MUST include, per target:

- `skill_id`
- `source_path`
- `target_path`
- `install_mode`
- `action` in `{create, update, noop, conflict}`
- `reasons` array

`plan` MUST be deterministic for same config + same filesystem state.

### 3.2 `apply`

Purpose: execute planned actions idempotently.

Rules:

- MUST clone/update source repositories into configured storage root before executing install mutations.
- MUST execute only `create`/`update` actions from resolved plan.
- MUST NOT mutate entries marked `conflict` unless `--force`.
- MUST run verification when `verify.enabled=true`.
- Re-running `apply` with unchanged state MUST produce only `noop`.

### 3.3 `doctor`

Purpose: inspect current state and report drift or risk.

Doctor checks MUST include:

- missing target path
- broken symlink
- target points to unexpected source
- source path missing
- verify-check failures

Doctor MUST report issue code, severity, and remediation hint.

#### 3.3.1 Doctor JSON Schema (`--json`)

When `--json` is set, `doctor` MUST emit a single JSON object to stdout with:

- `summary` object:
  - `total`: number (integer)
  - `error`: number (integer)
  - `warning`: number (integer)
- `findings` array of objects, where each finding MUST include:
  - `code`: string (stable machine code, recommended format: `UPPER_SNAKE_CASE`)
  - `severity`: string in `{error, warning}`
  - `skill_id`: string
  - `target_path`: string
  - `message`: string (human-readable)
  - `remediation`: string (human-readable hint)

The JSON output MUST be backwards compatible:

- Adding new optional fields is allowed.
- Removing or renaming required fields is not allowed.

### 3.4 `repair`

Purpose: reconcile recoverable drift discovered by doctor/plan.

Repair MUST attempt:

- recreate missing symlink/copy target
- relink broken symlink to expected source
- restore stale target mapping to config-defined mapping

Repair MUST NOT delete unknown user files unless `--force`.

## 4. Planned Config Lifecycle Commands

These commands are RECOMMENDED for post-Phase-1 CLI UX and may be implemented incrementally.

### 4.1 `init`

- MUST create default config at `~/.config/eden-skills/skills.toml` when absent.
- MUST fail safely unless `--force` when file already exists.

### 4.2 `add`

- MUST append a skill entry with required fields.
- MUST validate resulting config before persisting.

### 4.3 `remove <skill_id>`

- MUST remove the matching skill entry only.
- MUST error if `skill_id` does not exist.

### 4.4 `set <skill_id> ...`

- MUST mutate only targeted fields.
- MUST preserve file validity and deterministic structure.

### 4.5 `list`

- SHOULD display skill inventory and key metadata.

### 4.6 `config export`

- SHOULD emit full normalized TOML config.

### 4.7 `config import`

- MUST validate imported config before replacing current one.
- MUST support non-destructive preview mode (`--dry-run`) before write.

## 5. Exit Codes

- `0`: success (including no-op success)
- `1`: runtime failure (IO, permissions, git operation)
- `2`: config/schema validation error
- `3`: drift/conflict detected in strict mode

## 6. Logging

All commands SHOULD emit:

- summary counts (`create/update/noop/conflict/error`)
- per-target reasons on non-noop actions
- stable machine codes in `--json` mode
