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
