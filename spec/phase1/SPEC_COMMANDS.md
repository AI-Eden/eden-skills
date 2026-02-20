# SPEC_COMMANDS.md

Normative command contract for `eden-skills` CLI.

## 1. Global Flags

- `--config <path>`: config file path, default `~/.eden-skills/skills.toml`
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

#### 3.1.1 Plan JSON Schema (`--json`)

When `--json` is set, `plan` MUST emit a single JSON array to stdout.
Each array entry MUST be a JSON object with:

- `skill_id`: string
- `source_path`: string
- `target_path`: string
- `install_mode`: string in `{symlink, copy}`
- `action`: string in `{create, update, noop, conflict}`
- `reasons`: array of strings

The JSON output MUST be backwards compatible:

- Adding new optional fields is allowed.
- Removing or renaming required fields is not allowed.

#### 3.1.2 Copy Mode Delta Detection (Edge Cases)

When `install.mode=copy`, the plan engine MUST compare source and target content to decide between `noop` and `update`.

Rules:

- The comparison MUST use a streaming strategy for files (it MUST NOT require loading entire files into memory).
- The comparison MUST NOT follow symlinks. If a symlink is encountered anywhere in the source or target tree, the plan item MUST be marked `conflict`.
- If the comparison fails due to IO errors (permissions, unreadable paths, transient filesystem errors), the plan item MUST be marked `conflict` (the command MUST NOT abort the entire plan).
- For copy-comparison conflicts, `reasons` MUST include a stable message of the form: `copy comparison failed: <cause>`.
- `<cause>` MUST be one of: `permission denied`, `not found`, `symlink in tree`, `io error`.

### 3.2 `apply`

Purpose: execute planned actions idempotently.

Rules:

- MUST clone/update source repositories into configured storage root before executing install mutations.
- MUST execute only `create`/`update` actions from resolved plan.
- MUST NOT mutate entries marked `conflict` unless `--force`.
- MUST run verification when `verify.enabled=true`.
- Re-running `apply` with unchanged state MUST produce only `noop`.

#### 3.2.1 Source Sync Reporting and Failure Contract (`apply` and `repair`)

Source sync behavior MUST be deterministic and skill-scoped.

Rules:

- Source sync MUST attempt every configured skill in config order (it MUST NOT fail-fast on first skill error).
- Commands MUST emit a single source sync summary line in text mode:
  - `source sync: cloned=<n> updated=<n> skipped=<n> failed=<n>`
- `updated` MUST count skills whose synced repo `HEAD` changed during sync.
- `skipped` MUST count skills that were already present and remained on the same `HEAD` after sync.
- `failed` MUST count skills whose source sync failed at clone/fetch/checkout stages.
- For mixed outcomes in one run, summary counters MUST include both successful and failed skills.
- When `failed > 0`, `apply`/`repair` MUST fail with exit code `1` before target mutation and verification.
- When multiple source sync failures occur, diagnostics MUST include one entry per failed skill in config order.
- In `--strict` mode, source sync failure handling MUST take precedence over strict conflict exit behavior:
  - if `failed > 0`, command MUST return exit code `1`;
  - strict conflict exit code `3` applies only when `failed = 0`.
- Failure diagnostics MUST include actionable context for each failed skill:
  - `skill=<skill_id>`
  - `stage=<clone|fetch|checkout>`
  - `repo_dir=<resolved_storage_repo_path>`
  - `detail=<git_error_summary>`

### 3.3 `doctor`

Purpose: inspect current state and report drift or risk.

Doctor checks MUST include:

- missing target path
- broken symlink
- target points to unexpected source
- source path missing
- verify-check failures

Doctor MUST report issue code, severity, and remediation hint.

#### 3.3.1 Strict Mode Output Parity

When findings exist, `doctor --strict` MUST emit the same findings payload as non-strict mode for the same input state.

- In text mode, stdout finding lines MUST be equivalent; only process exit behavior differs.
- In JSON mode, stdout JSON payload MUST be equivalent; only process exit behavior differs.
- `--strict` changes exit semantics only (conflict exit), not finding content.

#### 3.3.2 Doctor JSON Schema (`--json`)

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

#### 3.4.1 Strict Conflict vs Verification Precedence (`apply` and `repair`)

When source sync has no failures (`failed = 0`), strict conflict handling and verification handling MUST follow one consistent precedence for both `apply` and `repair`:

- In `--strict` mode, if effective conflict count is greater than `0`, command MUST return strict conflict exit code `3` before post-mutation verification checks.
- If strict conflict condition is not met, verification failures MUST return runtime exit code `1`.
- Effective conflict count MUST exclude no-exec skills (`safety.no_exec_metadata_only=true`) as defined in section `3.5.3`.

### 3.5 Safety Gate MVP (Mechanics)

Phase 1 safety mechanics MUST cover license/status metadata, risk labeling, and no-exec behavior.

#### 3.5.1 Safety Metadata Persistence (`apply` and `repair`)

For every skill, after source sync, implementations MUST persist a deterministic metadata file under the synced repo root:

- path: `<storage_root>/<skill_id>/.eden-safety.toml`
- required fields:
  - `version` (`1`)
  - `skill_id`
  - `repo_path`
  - `source_path`
  - `retrieved_at_unix` (unix seconds)
  - `license_status` in `{permissive, non-permissive, unknown}`
  - `risk_labels` (array of strings)
  - `no_exec_metadata_only` (boolean)
- optional fields:
  - `license_hint`
  - `commit_sha`

#### 3.5.2 License and Risk Detection (Mechanics)

Safety detection MUST be deterministic for a given filesystem state.

- License detection MUST inspect repository license files and classify status to `{permissive, non-permissive, unknown}`.
- Risk labeling MUST scan source content and include stable labels for detected executable/script risk indicators.
- Detection failures MUST degrade to `unknown`/empty labels where possible instead of failing the command.

#### 3.5.3 No-Exec Metadata-Only Behavior

When `skills.safety.no_exec_metadata_only=true`:

- `apply` and `repair` MUST still sync source repositories.
- `apply` and `repair` MUST persist safety metadata.
- `apply` and `repair` MUST NOT mutate install targets for that skill (create/update operations are skipped).
- Verification checks for that skill MUST be skipped.
- In mixed-skill configs, verification for skills with `no_exec_metadata_only=false` MUST still run normally.
- Plan conflicts for `no_exec_metadata_only=true` skills MUST NOT contribute to strict conflict exits (`exit code 3`) in `apply` or `repair`.

#### 3.5.4 Doctor Safety Findings

`doctor` MUST emit safety findings in the same finding contract (`code/severity/message/remediation`):

- `LICENSE_NON_PERMISSIVE` warning
- `LICENSE_UNKNOWN` warning
- `RISK_REVIEW_REQUIRED` warning
- `NO_EXEC_METADATA_ONLY` warning (when enabled)

## 4. Planned Config Lifecycle Commands

These commands are RECOMMENDED for post-Phase-1 CLI UX and may be implemented incrementally.

### 4.1 `init`

- MUST create default config at `~/.eden-skills/skills.toml` when absent.
- MUST fail safely unless `--force` when file already exists.

### 4.2 `add`

CLI shape:

- `eden-skills add --config <path> --id <skill_id> --repo <git_url> --target <target_spec>...`

Supported flags:

- `--ref <ref>` (default: `main`)
- `--subpath <subpath>` (default: `.`)
- `--mode <symlink|copy>` (default: `symlink`)
- `--verify-enabled <true|false>` (default: `true`)
- `--verify-check <check>...` (default: mode-dependent checks from `SPEC_SCHEMA.md`)
- `--no-exec-metadata-only <true|false>` (default: `false`)

`target_spec` MUST be one of:

- `claude-code`
- `cursor`
- `custom:<path>` (path is required for custom targets)

Behavior:

- MUST load and validate the current config at `--config`.
- MUST error if `skill_id` already exists in config.
- MUST append exactly one new `[[skills]]` entry at the end of the skills array.
- MUST validate the resulting config before persisting.
- MUST write the normalized TOML form back to `--config`.

### 4.3 `remove <skill_id>`

CLI shape:

- `eden-skills remove <skill_id> --config <path>`

Behavior:

- MUST load and validate the current config at `--config`.
- MUST remove the matching skill entry only.
- MUST error if `skill_id` does not exist.
- MUST validate the resulting config before persisting.
- MUST write the normalized TOML form back to `--config`.

### 4.4 `set <skill_id> ...`

CLI shape:

- `eden-skills set <skill_id> --config <path> [flags...]`

Supported flags (all optional; at least one MUST be provided):

- `--repo <git_url>`
- `--ref <ref>`
- `--subpath <subpath>`
- `--mode <symlink|copy>`
- `--verify-enabled <true|false>`
- `--verify-check <check>...` (replaces the full checks list)
- `--target <target_spec>...` (replaces the full targets list; same `target_spec` rules as `add`)
- `--no-exec-metadata-only <true|false>`

Behavior:

- MUST load and validate the current config at `--config`.
- MUST error if `skill_id` does not exist.
- MUST error if no mutation flags are provided.
- MUST mutate only fields explicitly set by the user.
- MUST validate the resulting config before persisting.
- MUST write the normalized TOML form back to `--config`.

### 4.5 `list`

- SHOULD display skill inventory and key metadata.

#### 4.5.1 List JSON Schema (`--json`)

When `--json` is set, `list` MUST emit a single JSON object to stdout with:

- `count`: number (integer)
- `skills`: array of objects, where each skill object MUST include:
  - `id`: string
  - `source` object:
    - `repo`: string
    - `ref`: string
    - `subpath`: string
  - `install` object:
    - `mode`: string in `{symlink, copy}`
  - `verify` object:
    - `enabled`: boolean
    - `checks`: array of strings
  - `targets`: array of objects, where each target object MUST include:
    - `agent`: string in `{claude-code, cursor, custom}`
    - `path`: string (resolved path string; implementations MAY encode an error message as a string for unresolved paths)

The JSON output MUST be backwards compatible:

- Adding new optional fields is allowed.
- Removing or renaming required fields is not allowed.

### 4.6 `config export`

- SHOULD emit full normalized TOML config.

### 4.7 `config import`

- MUST validate imported config before replacing current one.
- MUST support non-destructive preview mode (`--dry-run`) before write.

#### 4.7.1 CLI Shape (Phase 1+)

`config import` MUST support:

- `--from <path>`: source config TOML path to import
- `--config <path>`: destination config path (default `~/.eden-skills/skills.toml`)
- `--dry-run`: do not write; instead emit normalized TOML to stdout

Validation rules:

- Imported config MUST be validated using the same schema rules as normal loading.
- In `--strict` mode, unknown top-level keys MUST be treated as errors.

Write rules:

- When not `--dry-run`, command MUST write the normalized TOML to destination path, replacing existing contents.

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
