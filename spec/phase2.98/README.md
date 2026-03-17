# Phase 2.98 Specifications: List Source Display, Doctor UX & Verify Dedup

**Status:** DRAFT
**Parent:** `spec/README.md`
**Planned by:** Architect (Claude Opus 4.6), 2026-03-17

## Purpose

Phase 2.98 addresses three UX and reliability improvements identified
after Phase 2.97 closed:

1. **List source display** — the `list` table's `Path` column shows a
   low-level repo-cache path that is meaningless to users. Replace it
   with a human-friendly `Source` column using the same
   `owner/repo (subpath)` format already rendered by
   `install --dry-run`.
2. **Doctor UX enhancement** — three improvements to `doctor` output:
   a. Add `--no-warning` flag to filter warning-severity findings.
   b. Rename the summary table header `Sev` → `Level` and the cell
      value `warn` → `warning` for consistency.
   c. Colorize the `Level` column: red for `error`, yellow for
      `warning`, dim for `info`.
3. **Verify deduplication** — when a symlink target is removed, the
   three default checks (`path-exists`, `is-symlink`,
   `target-resolves`) each independently report the same root cause.
   Short-circuit dependent checks when `path-exists` fails to
   eliminate redundant findings.

## Relationship to Earlier Phases

- Phase 1/2/2.5/2.7/2.8/2.9/2.95/2.97 specs are frozen.
- Phase 2.98 specs in this directory:
  1. **Replace** the Phase 2.97 `SPEC_TABLE_STYLE.md` Section 6.1
     `Path` column with a `Source` column that reuses
     `abbreviate_repo_url` + `abbreviate_home_path` from `ui/format.rs`.
  2. **Extend** the Phase 2.97 `doctor` command with a `--no-warning`
     filter flag and improved severity display.
  3. **Fix** the Phase 1 `verify_config_state` check loop to
     short-circuit when the target path does not exist.

## Scope Exclusions

- No Phase 3 features (crawler, taxonomy, curation).
- No changes to `skills.toml` format.
- No changes to `skills.lock` format.
- No changes to exit code semantics (0/1/2/3).
- No changes to `--json` output schemas for existing commands
  (new `--no-warning` flag only affects human and JSON filtering).

## Work Packages

| WP | Priority | Spec File | Domain | Description |
| :--- | :--- | :--- | :--- | :--- |
| WP-1 | **P0** | `SPEC_LIST_SOURCE.md` | CLI | Replace `Path` column with `Source` in `list` |
| WP-2 | **P0** | `SPEC_DOCTOR_UX.md` | CLI | `--no-warning` flag, Level rename, severity coloring |
| WP-3 | **P0** | `SPEC_VERIFY_DEDUP.md` | Core | Short-circuit verify checks when target is missing |
| WP-4 | **P1** | — | Docs | `README.md` + `docs/` update (closeout task) |
| -- | -- | `SPEC_TEST_MATRIX.md` | Testing | Phase 2.98 acceptance test scenarios |
| -- | -- | `SPEC_TRACEABILITY.md` | Traceability | Requirement-to-implementation mapping |

## Requirement ID Ranges

| Domain | ID Range |
| :--- | :--- |
| List Source | LSR-001 ~ LSR-003 |
| Doctor UX | DUX-001 ~ DUX-006 |
| Verify Dedup | VDD-001 ~ VDD-003 |
| Documentation | DOC-001 |
| Test Scenarios | TM-P298-001 ~ TM-P298-020 |

## Execution Order

```text
B1 (List Source / WP-1) ─────────────┐
B2 (Doctor UX / WP-2) ──────────────┤──→ B4 (Docs / WP-4) ──→ B5 (Regression)
B3 (Verify Dedup / WP-3) ───────────┘
```

B1–B3 are independent of each other.
B4 (documentation) depends on all preceding batches.
B5 (regression) depends on B4.

## New CLI Elements

| Element | Type | Description |
| :--- | :--- | :--- |
| `doctor --no-warning` | Flag | Hide warning-severity findings from output |

## Dependency Changes

None. All required crate features are already enabled.

## Normative Language

Same as `spec/README.md`: `MUST`, `SHOULD`, `MAY` per RFC 2119.
