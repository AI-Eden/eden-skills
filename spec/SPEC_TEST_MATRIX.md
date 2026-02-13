# SPEC_TEST_MATRIX.md

Minimum acceptance test matrix for Phase 1 CLI.

## 1. Environments

- Linux (latest stable)
- macOS (latest stable)

## 2. Required Scenarios

1. Fresh install
2. Repeated apply (idempotency)
3. Broken symlink recovery
4. Source moved or missing
5. Copy mode verification
6. Invalid config validation errors
7. Permission-denied target path

## 3. Scenario Assertions

### Fresh install

- `plan` shows only `create`
- `apply` succeeds
- `doctor` reports zero errors

### Repeated apply

- second `plan` shows all `noop`
- second `apply` performs no mutation

### Broken symlink recovery

- `doctor` emits `BROKEN_SYMLINK`
- `repair` fixes mapping
- post-repair `doctor` is clean

### Source moved or missing

- `doctor` emits `SOURCE_MISSING`
- `repair` fails safely with explicit reason
- unknown files are untouched

### Copy mode verification

- `plan`/`apply` respects `install.mode=copy`
- `doctor` uses copy-appropriate checks

### Invalid config

- schema error returns exit code `2`
- error includes exact field path

### Permission denied

- `apply` returns exit code `1`
- partial operations are reported clearly

## 4. CI Gate (Phase 1)

A release candidate MUST pass:

- all scenario tests in Linux CI
- at least one macOS smoke run
- idempotency assertions for both `symlink` and `copy`

## 5. Incremental Safety Scenarios

These scenarios are recommended incremental coverage for Safety Gate MVP mechanics:

- `apply` with `no_exec_metadata_only=true` syncs source and writes safety metadata, but does not mutate targets.
- `doctor` emits safety warnings for license unknown/non-permissive and risk-review-required labels.
- safety metadata file (`.eden-safety.toml`) includes required fields and stable enums.

## 6. Incremental Doctor Contract Scenarios

These scenarios are recommended incremental coverage for doctor contract stability:

- `doctor --strict` and `doctor` emit equivalent findings payloads for identical input state (text mode parity).
- `doctor --strict --json` and `doctor --json` emit equivalent JSON payloads for identical input state.
- doctor JSON payload under mixed warning/error findings preserves required fields (`summary.total/error/warning`, finding `code/severity/skill_id/target_path/message/remediation`).

## 7. Incremental Source Sync Scenarios

These scenarios are recommended incremental coverage for source sync hardening and diagnostics:

- Repeated `apply` with unchanged upstream reports `source sync` summary with `skipped > 0` and `failed = 0`.
- `apply` after upstream commit advance reports `source sync` summary with `updated > 0`.
- Source clone failure reports exit code `1` and includes `skill`, `stage=clone`, and `repo_dir` diagnostics.
- Source fetch failure reports exit code `1` and includes `skill`, `stage=fetch`, and `repo_dir` diagnostics.
- Source checkout failure reports exit code `1` and includes `skill`, `stage=checkout`, and `repo_dir` diagnostics.
- Multi-skill source sync failure diagnostics preserve config-order reporting (`skill A` before `skill B` when configured in that order).
- Mixed source sync outcome (`cloned/skipped/updated` plus `failed`) reports all counters deterministically in one summary line.

## 8. Incremental Strict-Mode Interaction Scenarios

These scenarios are recommended incremental coverage for strict-mode interaction precedence:

- `apply --strict` with source sync failures MUST return exit code `1` (runtime), even when other skills could produce plan conflicts.
- `repair --strict` with source sync failures MUST return exit code `1` (runtime), even when other skills could produce plan conflicts.
