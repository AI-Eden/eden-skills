# PHASE2_BUILDER_REMAINING.md

Index of remaining Builder-owned closeout work for Phase 2.
This file is intentionally short and points to detailed sources.

## Remaining Work Items

### P2-CLOSE-001 (CI blocker): Windows TOML/file URL escaping parity in Phase 2 command tests

- **Owner:** Builder
- **Priority:** P0 (release gate)
- **Current signal:** Latest hosted CI run for `main` (`run_id: 22148985041`) failed on `windows-latest`.
- **Observed failure mode:** `crates/eden-skills-cli/tests/phase2_commands.rs` writes `file://C:\...` into TOML literals, causing parse errors (`invalid unicode 8-digit hex code`).
- **Required outcome:** Normalize/escape file URL writes in Phase 2 command tests so Linux/macOS/Windows all pass.
- **Verification:** Hosted CI rerun must pass on all matrix targets (`ubuntu-latest`, `macos-latest`, `windows-latest`).
- **Related sources:** `spec/phase2/SPEC_TEST_MATRIX.md` (Section 5 cross-platform), `spec/phase2/SPEC_TRACEABILITY.md`, `STATUS.yaml`, `EXECUTION_TRACKER.md`.

### P2-CLOSE-002 (traceability closure): Resolve remaining planned Phase 2 test scenarios

- **Owner:** Builder
- **Priority:** P1
- **Current signal:** `spec/phase2/SPEC_TRACEABILITY.md` still marks multiple scenarios as `planned`.
- **Open scenarios:** `TM-P2-003`, `TM-P2-004`, `TM-P2-015`, `TM-P2-020`, `TM-P2-024`, `TM-P2-027`, `TM-P2-028`, `TM-P2-029`, `TM-P2-030`.
- **Required outcome:** For each scenario, choose one explicit disposition:
  1. Implement + automate, then update traceability status to `implemented`; or
  2. Mark deferred with rationale and target milestone in Phase 2 tracking docs.
- **Verification:** No ambiguous ownership for remaining `planned` entries; status and tracker are consistent.
- **Related sources:** `spec/phase2/SPEC_TEST_MATRIX.md`, `spec/phase2/SPEC_TRACEABILITY.md`, `STATUS.yaml`, `EXECUTION_TRACKER.md`.

### P2-CLOSE-003 (documentation consistency): Align strategic/project status docs with current Phase 2 state

- **Owner:** Builder
- **Priority:** P1
- **Current signal:** `README.md` and `ROADMAP.md` still present stale checklist/status wording compared with `STATUS.yaml` and `EXECUTION_TRACKER.md`.
- **Required outcome:** Keep strategic intent unchanged, but ensure project state wording does not regress to "Phase 1 focus" for current execution reporting.
- **Verification:** `spec/`, `STATUS.yaml`, `EXECUTION_TRACKER.md`, `README.md`, and `ROADMAP.md` are mutually non-contradictory on current phase execution state.

## Notes

- Architect-owned strategy outputs (taxonomy/rubric/crawler RFC finalization) are intentionally excluded.
- Historical completion details for Batch 1~7 remain in `EXECUTION_TRACKER.md` and `STATUS.yaml`.
- If new Builder-owned Phase 2 closeout gaps are discovered, add a new `P2-CLOSE-###` item and link it here.
