# Phase 2.97 Specifications: Reliability, Interactive UX & Docker Safety

**Status:** DRAFT
**Parent:** `spec/README.md`
**Planned by:** Architect (Claude Opus), 2026-03-07

## Purpose

Phase 2.97 addresses reliability bugs, UX modernization, and operational
safety gaps identified after Phase 2.95:

1. **Update concurrency fix** — the `update` command creates parallel
   git fetch tasks per skill, but multiple skills sharing the same
   repo-level cache race on the `.git/shallow` file. Fix by
   deduplicating refresh tasks by `repo_cache_key`.
2. **Table content styling** — enable `comfy-table`'s `custom_styling`
   feature so ANSI-styled cell content does not break column alignment.
   Add bold+magenta Skill IDs, status-colored cells, and bold headers.
3. **Interactive selection UX** — replace text-input skill selection in
   `remove` and `install` with `dialoguer::MultiSelect` checkbox lists.
   Hovered items in `install` show inline descriptions. Remove the `*`
   wildcard feature from `remove` (MultiSelect supersedes it).
4. **Cache cleanup** — add a `clean` command to remove orphaned
   `.repos/` cache entries, a `--auto-clean` flag on `remove`, and a
   `doctor` orphan-cache finding.
5. **Docker management domain** — introduce a `.eden-managed` manifest
   in agent skill directories to track external vs local ownership,
   preventing accidental cross-contamination when a Docker container
   also runs eden-skills independently.
6. **Hint arrow spec sync** — align frozen specs with the current
   implementation: `~>` (magenta) replaces `→` (dimmed) as the hint
   prefix in all output.
7. **Documentation update** — after all implementation, update
   `README.md` and `docs/` to reflect new commands, changed
   interactions, and new flags.

## Relationship to Earlier Phases

- Phase 1/2/2.5/2.7/2.8/2.9/2.95 specs are frozen.
- Phase 2.97 specs in this directory:
  1. **Fix** the Phase 2.95 `update` Mode A refresh to deduplicate by
     repo cache key (same dedup pattern as `source.rs`).
  2. **Upgrade** the Phase 2.8 `comfy-table` integration with the
     `custom_styling` feature and content styling rules.
  3. **Replace** the Phase 2.7/2.95 text-input remove selection and
     the Phase 2.5 install selection with `MultiSelect` checkbox UX.
     **Remove** the Phase 2.95 `*` wildcard feature (`RMA-001~004`).
  4. **Add** a `clean` command and `--auto-clean` flag for repo-cache
     hygiene (new functionality, no existing spec dependency).
  5. **Add** `.eden-managed` manifest for Docker cross-container
     ownership tracking (extends Phase 2.95 Docker bind-mount model).
  6. **Amend** Phase 2.8 `SPEC_OUTPUT_UPGRADE.md` OUP-013 and Phase 2.9
     `SPEC_NEWLINE_POLICY.md` hint prefix from `→` (dimmed) to `~>`
     (magenta) to match the shipped implementation.
  7. **Update** `README.md` and `docs/` as a closeout task.

## Scope Exclusions

- No Phase 3 features (crawler, taxonomy, curation).
- No changes to `skills.toml` format.
- No changes to `skills.lock` format.
- No changes to exit code semantics (0/1/2/3).
- No changes to `--json` output schemas for existing commands
  (new `clean` command adds its own `--json` schema).

## Work Packages

| WP | Priority | Spec File | Domain | Description |
| :--- | :--- | :--- | :--- | :--- |
| WP-1 | **P0** | `SPEC_UPDATE_FIX.md` | CLI | Deduplicate Mode A refresh tasks by repo cache key |
| WP-2 | **P0** | `SPEC_TABLE_STYLE.md` | CLI | `custom_styling` feature + table content color rules |
| WP-3 | **P1** | `SPEC_INTERACTIVE_UX.md` | CLI | MultiSelect for remove + install, description-on-hover |
| WP-4 | **P1** | `SPEC_CACHE_CLEAN.md` | Core + CLI | `clean` command, `--auto-clean`, doctor orphan check |
| WP-5 | **P2** | `SPEC_DOCKER_MANAGED.md` | Core + CLI | `.eden-managed` manifest, ownership guard, doctor check |
| WP-6 | **P1** | `SPEC_HINT_SYNC.md` | Spec | Hint prefix amendment `→` → `~>` |
| WP-7 | **P1** | — | Docs | `README.md` + `docs/` update (closeout task) |
| -- | -- | `SPEC_TEST_MATRIX.md` | Testing | Phase 2.97 acceptance test scenarios |
| -- | -- | `SPEC_TRACEABILITY.md` | Traceability | Requirement-to-implementation mapping |

## Requirement ID Ranges

| Domain | ID Range |
| :--- | :--- |
| Update Fix | UFX-001 ~ UFX-003 |
| Table Style | TST-001 ~ TST-010 |
| Interactive UX | IUX-001 ~ IUX-010 |
| Cache Clean | CCL-001 ~ CCL-007 |
| Docker Managed | DMG-001 ~ DMG-008 |
| Hint Sync | HSY-001 ~ HSY-002 |
| Documentation | DOC-001 ~ DOC-002 |
| Test Scenarios | TM-P297-001 ~ TM-P297-065 |

## Execution Order

```text
B1 (Update Fix / WP-1) ──────────────────────────┐
B2 (Table Style / WP-2) ─────────────────────────┤
B3 (Hint Sync / WP-6) ───────────────────────────┤
B4 (Interactive UX / WP-3) ──────────────────────┤──→ B7 (Docs / WP-7) ──→ B8 (Regression)
B5 (Cache Clean / WP-4) ─────────────────────────┤
B6 (Docker Managed / WP-5) ──────────────────────┘
```

B1–B6 are independent of each other.
B7 (documentation) depends on all preceding batches.
B8 (regression) depends on B7.

## New CLI Elements

| Element | Type | Description |
| :--- | :--- | :--- |
| `clean` | Subcommand | Remove orphaned repo-cache entries |
| `remove --auto-clean` | Flag | Auto-clean orphaned cache after removal |
| `remove --force` | Flag (Docker context) | Force-remove externally-managed skills |
| `install --force` | Flag (Docker context) | Force-overwrite externally-managed skills |

## Dependency Changes

| Crate | Change | Purpose |
| :--- | :--- | :--- |
| `comfy-table` | `"7"` → `{ version = "7", features = ["custom_styling"] }` | ANSI-safe column width calculation |

## Normative Language

Same as `spec/README.md`: `MUST`, `SHOULD`, `MAY` per RFC 2119.
