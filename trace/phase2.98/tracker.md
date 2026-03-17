# Phase 2.98 Execution Tracker

Phase: List Source Display, Doctor UX & Verify Dedup
Status: Closeout Completed
Started: 2026-03-17
Completed: 2026-03-17

## Batch Plan

| Batch | Name | WP | Requirements | Status |
| --- | --- | --- | --- | --- |
| 1 | All Implementation (List Source + Doctor UX + Verify Dedup) | WP-1 + WP-2 + WP-3 | LSR-001~003, DUX-001~006, VDD-001~003 | completed |
| 2 | Documentation + Regression + Closeout | WP-4 | DOC-001 | completed |

## Dependency Constraints

- Batch 1 is independent.
- Batch 2 (documentation + regression) depends on Batch 1.

## Completion Records

### Batch 1 — All Implementation (List Source + Doctor UX + Verify Dedup) (Completed 2026-03-17)

- Requirements: `LSR-001`, `LSR-002`, `LSR-003`, `DUX-001`, `DUX-002`, `DUX-003`, `DUX-004`, `DUX-005`, `DUX-006`, `VDD-001`, `VDD-002`, `VDD-003`
- Completed in this pass:
  - Updated `crates/eden-skills-core/src/verify.rs` so missing target paths short-circuit non-`path-exists` checks, reducing duplicate verify/doctor findings while preserving the existing check behavior for present targets.
  - Updated `crates/eden-skills-cli/src/commands/config_ops.rs` to replace the human `list` table `Path` column with a `Source` column rendered as `owner/repo (subpath)` or `~/local-path (subpath)` using cyan styling, while leaving `list --json` unchanged.
  - Added `DoctorArgs` in `crates/eden-skills-cli/src/lib.rs`, wired the new `doctor --no-warning` flag through command dispatch, and updated `crates/eden-skills-cli/src/commands/diagnose.rs` to filter warning findings before both human/JSON rendering and strict exit evaluation.
  - Renamed the doctor summary-table severity column from `Sev` to `Level`, expanded its width for `warning`, and colorized `error` / `warning` / `info` values red / yellow / dim to match the existing card-level severity semantics.
  - Added `crates/eden-skills-cli/tests/list_source_tests.rs`, `crates/eden-skills-cli/tests/doctor_ux_tests.rs`, `crates/eden-skills-cli/tests/verify_dedup_tests.rs`, and `crates/eden-skills-core/tests/verify_dedup_tests.rs`, and refreshed superseded legacy assertions in the affected CLI test files so historical coverage now matches the Phase 2.98 contracts.
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `476`
- Notes:
  - `doctor --no-warning` intentionally remains behavior-neutral for exit codes and JSON schema beyond filtering warning findings out of the existing payload.
  - The Batch 1 regression pass stayed within Phase 2.98 scope only; root `STATUS.yaml`, root `EXECUTION_TRACKER.md`, and end-user docs remain untouched until Batch 2 closeout.

### Batch 2 — Documentation + Regression + Closeout (Completed 2026-03-17)

- Requirements: `DOC-001`
- Completed in this pass:
  - Updated `README.md` to mention `doctor --no-warning`, clarify that `list` shows source origins, refresh the Phase status line through 2.98, and sync the headline test/spec counts.
  - Updated `docs/07-cli-reference.md` to document the human `list` `Source` column semantics plus the new `doctor --no-warning` option, while explicitly noting that doctor JSON output keeps the existing schema.
  - Updated `docs/06-troubleshooting.md` with `doctor --no-warning` usage, a `Level`-column example, and an error-focused JSON command example for automation.
  - Completed Phase 2.98 closeout by marking `DOC-001` complete in `spec/phase2.98/SPEC_TRACEABILITY.md`, updating `trace/phase2.98/status.yaml` and `trace/phase2.98/tracker.md`, and synchronizing root `STATUS.yaml` and `EXECUTION_TRACKER.md`.
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace --all-targets` ✅
  - `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` ✅
  - JSON output contracts unchanged ✅
  - Exit code semantics unchanged ✅
  - Test inventory: `476`
- Notes:
  - Phase 2.98 now closes with both batches complete; no additional implementation or documentation work remains in this phase.
