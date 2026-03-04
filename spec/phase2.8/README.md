# Phase 2.8 Specifications: TUI Deep Optimization & Code Maintainability

**Status:** DRAFT
**Parent:** `spec/README.md`
**Planned by:** Architect (Claude Opus), 2026-03-04

## Purpose

Phase 2.8 addresses the visual and structural gaps remaining after
Phase 2.5/2.7. It delivers three outcomes:

1. **Output completeness** — every human-mode command output is upgraded
   to use `UiContext`, colored symbols, and action prefixes, closing the
   gap between the Phase 2.5 visual design language and actual
   implementation.
2. **Table rendering** — structured, multi-record outputs (`list`,
   `plan`, `doctor`, `install --dry-run`, etc.) are rendered as
   terminal-aware tables via `comfy-table`.
3. **Code maintainability** — the monolithic `commands.rs` (~3 768 lines)
   is decomposed into focused sub-modules, and critical doc comments are
   added across both CLI and Core crates.

## Relationship to Earlier Phases

- Phase 1 specs (`spec/phase1/`) are frozen.
- Phase 2 specs (`spec/phase2/`) are frozen.
- Phase 2.5 specs (`spec/phase2.5/`) are frozen.
- Phase 2.7 specs (`spec/phase2.7/`) are frozen.
- Phase 2.8 specs in this directory:
  1. **Extend** the Phase 2.5 `SPEC_CLI_UX.md` visual design language
     with table rendering and upgraded output for all commands.
  2. **Fulfill** the remaining gap between Phase 2.5/2.7 spec-defined
     ideal output and actual implementation.
  3. **Do NOT modify** any existing command semantics, exit codes,
     `--json` output contracts, `skills.toml` format, or `skills.lock`
     format.

## Scope Exclusions

- No Phase 3 features (crawler, taxonomy, curation).
- No new CLI commands or flags.
- No changes to `--json` output schemas.
- No changes to exit code semantics (1/2/3).
- No changes to `skills.toml` or `skills.lock` format.
- No performance optimization work.

## Work Packages

| WP | Priority | Spec File | Domain | Description |
| :--- | :--- | :--- | :--- | :--- |
| WP-1 | **P0** | `SPEC_TABLE_RENDERING.md` | CLI | Table library, column definitions, long-content strategy, degradation rules |
| WP-2 | **P0** | `SPEC_OUTPUT_UPGRADE.md` | CLI | Full-command output upgrade (Category A + B), UiContext unification, error format |
| WP-3 | **P1** | `SPEC_CODE_STRUCTURE.md` | CLI + Core | `commands.rs` decomposition, doc comment coverage |
| -- | -- | `SPEC_TEST_MATRIX.md` | Testing | Phase 2.8 acceptance test scenarios |
| -- | -- | `SPEC_TRACEABILITY.md` | Traceability | Requirement-to-implementation mapping |

## Requirement ID Ranges

| Domain | ID Range |
| :--- | :--- |
| Table Rendering | TBL-001 ~ TBL-007 |
| Output Upgrade | OUP-001 ~ OUP-020 |
| Code Structure | CST-001 ~ CST-008 |
| Test Scenarios | TM-P28-001 ~ TM-P28-040 |

## Execution Order

```text
WP-1 (Table Rendering) ────┐
                            ├──→ WP-2 (Output Upgrade)
WP-3a (commands.rs split) ──┘         │
                                      │
WP-3b (doc comments) ◄───────────────┘
```

WP-3a (`commands.rs` decomposition) is a pure refactoring prerequisite
that SHOULD be executed before WP-2 to reduce diff complexity. WP-1
provides the table infrastructure that WP-2 depends on. WP-3b (doc
comments) is performed alongside and after WP-2 as modules stabilize.

## Normative Language

Same as `spec/README.md`: `MUST`, `SHOULD`, `MAY` per RFC 2119.
