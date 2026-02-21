# Phase 2.7 Specifications: UX Polish & Lock File

**Status:** DRAFT (Pending Builder Implementation)
**Parent:** `spec/README.md`
**Planned by:** Architect (Claude Opus), 2026-02-21

## Purpose

Phase 2.7 addresses user-experience gaps identified after the Phase 2.5 MVP
launch. It introduces no new feature domains but systematically polishes the
CLI surface across four axes:

1. **State correctness** — `skills.lock` enables diff-driven reconciliation
   and eliminates orphan files when skills are removed from `skills.toml`.
2. **Help quality** — every command and argument gains a human-readable
   description; `--version` is added.
3. **Output quality** — hardcoded ANSI escape codes are replaced with
   `owo-colors`; error messages are refined with contextual detail.
4. **Remove ergonomics** — batch removal and interactive selection.

## Relationship to Earlier Phases

- Phase 1 specs (`spec/phase1/`) are frozen.
- Phase 2 specs (`spec/phase2/`) are frozen.
- Phase 2.5 specs (`spec/phase2.5/`) are frozen.
- Phase 2.7 specs in this directory:
  1. **Amend** the Phase 2.5 `SPEC_CLI_UX.md` technology stack (replace
     `console` with `owo-colors`).
  2. **Extend** `plan.rs` `Action` enum with a `Remove` variant, gated by
     lock-file awareness.
  3. **Do NOT modify** any existing command semantics. All backward
     compatibility contracts remain intact.

## Work Packages

| WP | Priority | Spec File | Domain | Description |
| :--- | :--- | :--- | :--- | :--- |
| WP-1 | **P0** | `SPEC_LOCK.md` | Core | `skills.lock` format, lifecycle, and diff-driven reconciliation |
| WP-2 | **P0** | `SPEC_HELP_SYSTEM.md` | CLI | Help text, version info, command grouping, short flags |
| WP-3 | **P0** | `SPEC_OUTPUT_POLISH.md` | CLI | `owo-colors` migration, error refinement, `--color` flag |
| WP-4 | **P1** | `SPEC_REMOVE_ENH.md` | CLI | Batch remove, interactive selection, install UX flags |
| -- | -- | `SPEC_TEST_MATRIX.md` | Testing | Phase 2.7 acceptance test scenarios |
| -- | -- | `SPEC_TRACEABILITY.md` | Traceability | Requirement-to-implementation mapping |

## Requirement ID Ranges

| Domain | ID Range |
| :--- | :--- |
| Lock File | LCK-001 ~ LCK-010 |
| Help System | HLP-001 ~ HLP-007 |
| Output Polish | OUT-001 ~ OUT-008 |
| Remove Enhancements | RMV-001 ~ RMV-005 |
| Test Scenarios | TM-P27-001 ~ TM-P27-040 |

## Dependency Graph

```text
WP-1 (Lock File) ──────→ apply/plan/repair now support Remove actions
        │
        │        WP-2 (Help System) ──┐
        │                             ├──→ README / docs refresh (non-spec)
        │        WP-3 (Output Polish)─┘
        │                │
        └────────────────┴──→ WP-4 (Remove Enhancements)
```

WP-1 is a prerequisite for WP-4 (lock file enables diff-based batch removal
awareness in `apply`). WP-2 and WP-3 are independent of each other and of
WP-1; they may be implemented in parallel.

## Normative Language

Same as `spec/README.md`: `MUST`, `SHOULD`, `MAY` per RFC 2119.
