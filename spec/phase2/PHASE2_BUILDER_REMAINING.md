# PHASE2_BUILDER_REMAINING.md

Index of Builder-owned closeout work for Phase 2.
This file remains concise and points to detailed status sources.

## Closeout Snapshot

- **Scope:** `P2-CLOSE-001` through `P2-CLOSE-003`
- **State:** In progress (implementation complete; hosted CI matrix verification pending push/rerun)
- **Detailed sources:** `spec/phase2/SPEC_TRACEABILITY.md`, `STATUS.yaml`, `EXECUTION_TRACKER.md`

## Completed Closeout Items

### P2-CLOSE-001 (CI blocker): Windows TOML/file URL escaping parity in Phase 2 command tests

- **Status:** Implemented + locally validated (hosted CI matrix verification pending)
- **Outcome:** `crates/eden-skills-cli/tests/phase2_commands.rs` now normalizes file URLs to TOML-safe format before writing literals, and adds a regression test for Windows-style `file://C:\...` payload parsing.
- **Verification:** local Phase 2 command suite passes; workspace quality gate recorded in tracker and status docs; hosted matrix validation requires a run that includes current commit.

### P2-CLOSE-002 (traceability closure): Resolve remaining planned Phase 2 test scenarios

- **Status:** Completed (all prior `planned` entries explicitly dispositioned)
- **Implemented now:** `TM-P2-003`, `TM-P2-004`, `TM-P2-020`, `TM-P2-024`, `TM-P2-028`, `TM-P2-030`
- **Deferred with milestone:** `TM-P2-015`, `TM-P2-027`, `TM-P2-029` -> `phase2_post_release_hardening`

### P2-CLOSE-003 (documentation consistency): Align strategic/project status docs with current Phase 2 state

- **Status:** Completed
- **Outcome:** status wording/checklists synced across project tracking docs so Phase 2 execution state no longer regresses to Phase 1-focused phrasing.

## Remaining Builder-Owned Closeout Work

- Re-run hosted CI matrix (`ubuntu-latest`, `macos-latest`, `windows-latest`) on a commit containing the `P2-CLOSE-001` fix and confirm green status.

## Deferred Follow-Ups (Post-Closeout)

- `TM-P2-015`: explicit warning-channel contract for Docker symlink fallback output (`phase2_post_release_hardening`)
- `TM-P2-027`: deterministic Windows privilege-denied symlink automation harness (`phase2_post_release_hardening`)
- `TM-P2-029`: dedicated Windows-hosted safety graceful-degradation assertions (`phase2_post_release_hardening`)

## Notes

- Architect-owned strategy outputs (taxonomy/rubric/crawler RFC finalization) remain out of Builder closeout scope.
- Historical completion details for Batch 1~7 remain in `EXECUTION_TRACKER.md` and `STATUS.yaml`.
