# PHASE1_BUILDER_REMAINING.md

Index of remaining Builder-owned work for Phase 1 CLI.
This file is intentionally short and points to detailed sources.

## Remaining Work Items

| ID | Task | Detail References |
|---|---|---|
| B-026 | Refresh hosted CI verification on latest `main`-equivalent changes (`fmt` + `clippy` + `test` workflow gates) and record run metadata. | `.github/workflows/ci.yml`, `STATUS.yaml` (`phase1.ci.hosted_run_reference`), `EXECUTION_TRACKER.md` (Section `6.1`) |
| B-027 | Run Phase 1 closeout audit for Builder scope: command behavior/spec parity, traceability completeness, and test-matrix coverage consistency. | `spec/SPEC_COMMANDS.md`, `spec/SPEC_TEST_MATRIX.md`, `spec/SPEC_TRACEABILITY.md`, `EXECUTION_TRACKER.md` (Section `6.1`) |

## Notes

- Architect-owned tasks (taxonomy, rubric, crawler strategy RFC) are intentionally excluded.
- If B-027 discovers functional gaps, add a new Builder task ID and link it here.
