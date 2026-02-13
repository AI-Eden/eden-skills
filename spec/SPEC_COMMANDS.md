# SPEC_COMMANDS.md

Normative command contract for `eden-skills` Phase 1.

## 1. Global Flags

- `--config <path>`: config file path, default `~/.config/eden-skills/skills.toml`
- `--json`: emit machine-readable output
- `--strict`: unknown keys and warnings become errors

## 2. `plan`

Purpose: compute dry-run action graph without mutating filesystem.

Output MUST include, per target:

- `skill_id`
- `source_path`
- `target_path`
- `install_mode`
- `action` in `{create, update, noop, conflict}`
- `reasons` array

`plan` MUST be deterministic for same config + same filesystem state.

## 3. `apply`

Purpose: execute planned actions idempotently.

Rules:

- MUST execute only `create`/`update` actions from resolved plan.
- MUST NOT mutate entries marked `conflict` unless `--force`.
- MUST run verification when `verify.enabled=true`.
- Re-running `apply` with unchanged state MUST produce only `noop`.

## 4. `doctor`

Purpose: inspect current state and report drift or risk.

Doctor checks MUST include:

- missing target path
- broken symlink
- target points to unexpected source
- source path missing
- verify-check failures

Doctor MUST report issue code, severity, and remediation hint.

## 5. `repair`

Purpose: reconcile recoverable drift discovered by doctor/plan.

Repair MUST attempt:

- recreate missing symlink/copy target
- relink broken symlink to expected source
- restore stale target mapping to config-defined mapping

Repair MUST NOT delete unknown user files unless `--force`.

## 6. Exit Codes

- `0`: success (including no-op success)
- `1`: runtime failure (IO, permissions, git operation)
- `2`: config/schema validation error
- `3`: drift/conflict detected in strict mode

## 7. Logging

All commands SHOULD emit:

- summary counts (`create/update/noop/conflict/error`)
- per-target reasons on non-noop actions
- stable machine codes in `--json` mode
