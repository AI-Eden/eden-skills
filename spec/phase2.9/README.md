# Phase 2.9 Specifications: UX Polish, Update Semantics & Output Consistency

**Status:** DRAFT
**Parent:** `spec/README.md`
**Planned by:** Architect (Claude Opus), 2026-03-05

## Purpose

Phase 2.9 addresses five user-facing quality gaps discovered during
production use of Phase 2.8:

1. **Table rendering fix** — eliminate the phantom empty column caused
   by `ContentArrangement::Dynamic` under `UTF8_FULL_CONDENSED`.
2. **Update command extension** — extend `update` to refresh Mode A
   (URL-installed) skill sources in addition to registry indexes,
   with an `--apply` flag for immediate reconciliation.
3. **Install UX overhaul** — card-style discovery preview, step-style
   progress bars for source sync, and tree-style hierarchical output
   for install results.
4. **Output consistency** — unify `add`, `set`, `config import`,
   `remove` candidates, and all remaining raw-format outputs to
   the Phase 2.5/2.8 visual design language. Introduce path coloring.
5. **Newline normalization** — establish and enforce a strict policy
   for trailing newlines, section spacing, and error formatting
   across all command output paths.

## Relationship to Earlier Phases

- Phase 1/2/2.5/2.7 specs are frozen.
- Phase 2.8 specs are frozen.
- Phase 2.9 specs in this directory:
  1. **Fix** the Phase 2.8 table rendering infrastructure (TBL-002
     override: `Dynamic` → `DynamicFullWidth` with column constraints).
  2. **Extend** the Phase 2 `update` command semantics beyond registry
     sync to cover Mode A skill source refresh.
  3. **Supersede** the Phase 2.8 install discovery output with a
     card-style list, step-style sync progress, and tree-style
     install results.
  4. **Complete** the output unification started in Phase 2.8 for
     all remaining commands.
  5. **Introduce** a project-wide newline normalization policy.

## Scope Exclusions

- No Phase 3 features (crawler, taxonomy, curation).
- No new CLI commands (all changes are to existing commands).
- No changes to `--json` output schemas.
- No changes to exit code semantics (0/1/2/3).
- No changes to `skills.toml` or `skills.lock` format.
- No new crate dependencies.

## Work Packages

| WP | Priority | Spec File | Domain | Description |
| :--- | :--- | :--- | :--- | :--- |
| WP-1 | **P0** | `SPEC_TABLE_FIX.md` | CLI | `DynamicFullWidth` migration, column constraint policy |
| WP-2 | **P0** | `SPEC_UPDATE_EXT.md` | CLI + Core | `update` dual-layer refresh (registries + Mode A skills), `--apply` flag |
| WP-3 | **P0** | `SPEC_INSTALL_UX.md` | CLI | Discovery card preview, step-style sync progress, tree-style results |
| WP-4 | **P0** | `SPEC_OUTPUT_CONSISTENCY.md` | CLI | `add`/`set`/`config import`/`remove` output upgrade, path coloring, UiContext gaps |
| WP-5 | **P0** | `SPEC_NEWLINE_POLICY.md` | CLI | Trailing newline elimination, section spacing, error format fix |
| -- | -- | `SPEC_TEST_MATRIX.md` | Testing | Phase 2.9 acceptance test scenarios |
| -- | -- | `SPEC_TRACEABILITY.md` | Traceability | Requirement-to-implementation mapping |

## Requirement ID Ranges

| Domain | ID Range |
| :--- | :--- |
| Table Fix | TFX-001 ~ TFX-003 |
| Update Extension | UPD-001 ~ UPD-008 |
| Install UX | IUX-001 ~ IUX-008 |
| Output Consistency | OCN-001 ~ OCN-010 |
| Newline Policy | NLP-001 ~ NLP-006 |
| Test Scenarios | TM-P29-001 ~ TM-P29-040 |

## Execution Order

```text
WP-1 (Table Fix) ──────────────┐
                                ├──→ WP-3 (Install UX)
WP-5 (Newline Policy) ─────────┤
                                ├──→ WP-4 (Output Consistency)
WP-2 (Update Extension) ───────┘
```

WP-1 (table fix) and WP-5 (newline policy) are infrastructure
prerequisites. WP-2 is independent. WP-3 and WP-4 depend on WP-1
for correct table rendering and on WP-5 for consistent spacing.

## New CLI Flag

| Flag | Command | Default | Description |
| :--- | :--- | :--- | :--- |
| `--apply` | `update` | off | After refreshing sources, also reconcile targets |

## Normative Language

Same as `spec/README.md`: `MUST`, `SHOULD`, `MAY` per RFC 2119.
