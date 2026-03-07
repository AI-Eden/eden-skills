# SPEC_TEST_MATRIX.md

Phase 2.97 acceptance test scenarios.

## 1. Convention

- Scenario IDs: `TM-P297-001` to `TM-P297-059`.
- Tests marked `auto` are implemented as Rust integration tests.
- Tests marked `manual` require manual verification.

## 2. Update Fix (WP-1)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P297-001 | Two skills from the same repo produce one git fetch during update | UFX-001 | auto | pending |
| TM-P297-002 | Update table shows correct per-skill status after deduplicated fetch | UFX-002 | auto | pending |
| TM-P297-003 | Consecutive `update` calls do not fail with "Another git process" error | UFX-003 | auto | pending |
| TM-P297-004 | Stale `.git/shallow.lock` older than 60s is removed before fetch | UFX-003 | auto | pending |
| TM-P297-005 | Local-source skills are not grouped (no repo cache) | UFX-001 | auto | pending |
| TM-P297-006 | `update --json` output contains correct per-skill status after dedup | UFX-002 | auto | pending |

## 3. Table Style (WP-2)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P297-007 | `Cargo.toml` contains `comfy-table` with `custom_styling` feature | TST-001 | auto | pending |
| TM-P297-008 | Table headers contain ANSI bold sequence when colors are enabled | TST-002 | auto | pending |
| TM-P297-009 | Skill ID cells contain ANSI bold+magenta when colors are enabled | TST-003 | auto | pending |
| TM-P297-010 | Status cells use green for `up-to-date`, red for `failed` | TST-004 | auto | pending |
| TM-P297-011 | Table with styled cells has consistent column widths (no misalignment) | TST-005 | auto | pending |
| TM-P297-012 | Non-TTY output contains no ANSI codes in table cells | TST-002 | auto | pending |
| TM-P297-057 | `eden-skills --help` shows bold green headers, bold cyan literals, magenta placeholders | TST-006 | auto | pending |
| TM-P297-058 | `list` table shows `Path` column (not `Source`) with repo-cache paths | TST-007 | auto | pending |
| TM-P297-059 | `list` Agents column truncates at 5 with `+N more` in yellow | TST-008 | auto | pending |

## 4. Interactive UX — Remove (WP-3)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P297-013 | `remove` without IDs in interactive mode shows checkbox selector with `...` overflow indicators | IUX-001 | auto | pending |
| TM-P297-014 | MultiSelect selection followed by confirmation removes selected skills | IUX-001, IUX-005 | auto | pending |
| TM-P297-015 | Confirmation declined (`N`) cancels removal | IUX-005 | auto | pending |
| TM-P297-016 | `EDEN_SKILLS_TEST_REMOVE_INPUT="0,2"` selects correct items | IUX-007 | auto | pending |
| TM-P297-017 | `EDEN_SKILLS_TEST_REMOVE_INPUT="interrupt"` cancels gracefully | IUX-007 | auto | pending |
| TM-P297-018 | Non-TTY remove without IDs produces argument error | IUX-008 | auto | pending |
| TM-P297-019 | `*` input is no longer recognized as wildcard | IUX-006 | auto | pending |
| TM-P297-020 | `remove skill-a skill-b` (explicit IDs) bypasses MultiSelect | IUX-008 | auto | pending |

## 5. Interactive UX — Install (WP-3)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P297-021 | Multi-skill discovery in interactive mode shows checkbox selector with `...` overflow indicators | IUX-002 | auto | pending |
| TM-P297-022 | Active unchecked item shows cyan checkbox and dim inline description without bold text | IUX-003, IUX-004, IUX-009 | auto | pending |
| TM-P297-023 | Checked item keeps inline description and truncates after 57 characters with `...` | IUX-003, IUX-004 | auto | pending |
| TM-P297-024 | Skill without description shows name only when hovered | IUX-010 | auto | pending |
| TM-P297-025 | `EDEN_SKILLS_TEST_SKILL_INPUT="0,1"` selects correct items | IUX-007 | auto | pending |
| TM-P297-026 | `--all` flag bypasses MultiSelect | IUX-008 | auto | pending |
| TM-P297-027 | `--skill <name>` flag bypasses MultiSelect | IUX-008 | auto | pending |
| TM-P297-028 | Single skill discovered installs directly without prompt | IUX-008 | auto | pending |

## 6. Cache Clean (WP-4)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P297-029 | `clean` removes orphaned `.repos/` entries not in config | CCL-001 | auto | pending |
| TM-P297-030 | `clean` removes stale `eden-skills-discovery-*` temp dirs | CCL-002 | auto | pending |
| TM-P297-031 | `clean --dry-run` lists removals without deleting | CCL-003 | auto | pending |
| TM-P297-032 | `clean --json` outputs machine-readable report | CCL-004 | auto | pending |
| TM-P297-033 | `clean` with no orphans reports nothing to clean | CCL-001 | auto | pending |
| TM-P297-034 | `remove --auto-clean` runs clean after removal | CCL-005 | auto | pending |
| TM-P297-035 | `doctor` reports `ORPHAN_CACHE_ENTRY` for orphaned cache | CCL-006 | auto | pending |
| TM-P297-036 | `clean` reports freed disk space in human mode | CCL-007 | auto | pending |

## 7. Docker Managed (WP-5)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P297-037 | Install to docker target writes `.eden-managed` with `source: "external"` | DMG-001, DMG-002 | auto | pending |
| TM-P297-038 | Local install writes `.eden-managed` with `source: "local"` | DMG-001, DMG-003 | auto | pending |
| TM-P297-039 | Remove of externally-managed skill defaults to config-only removal | DMG-004 | auto | pending |
| TM-P297-040 | `remove --force` of externally-managed skill deletes files and manifest entry | DMG-005 | auto | pending |
| TM-P297-041 | Install of existing externally-managed skill shows warning | DMG-006 | auto | pending |
| TM-P297-042 | `doctor` reports `DOCKER_OWNERSHIP_CHANGED` when manifest shows local takeover | DMG-007 | auto | pending |
| TM-P297-043 | `doctor` reports `DOCKER_EXTERNALLY_REMOVED` when skill is missing | DMG-007 | auto | pending |
| TM-P297-044 | Missing `.eden-managed` does not block operations | DMG-008 | auto | pending |
| TM-P297-045 | Corrupted `.eden-managed` emits warning and proceeds | DMG-008 | auto | pending |
| TM-P297-046 | `apply --force` reclaims ownership from local back to external | DMG-004 | auto | pending |

## 8. Hint Sync (WP-6)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P297-047 | Error hint uses `~>` prefix (not `→`) | HSY-001 | auto | pending |
| TM-P297-048 | `~>` is styled magenta when colors are enabled | HSY-002 | auto | pending |
| TM-P297-049 | Doctor remediation uses `~>` magenta prefix | HSY-001 | auto | pending |
| TM-P297-050 | Update guidance uses `~>` prefix | HSY-001 | auto | pending |

## 9. Documentation (WP-7)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P297-051 | `README.md` documents `clean` command | DOC-001 | manual | pending |
| TM-P297-052 | `README.md` documents `remove --auto-clean` flag | DOC-001 | manual | pending |
| TM-P297-053 | `docs/` updated with new interactive selection behavior | DOC-002 | manual | pending |
| TM-P297-054 | `README.md` Supported Agents table up to date | DOC-001 | manual | pending |

## 10. Regression

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P297-055 | `cargo fmt --all -- --check` passes | — | auto | pending |
| TM-P297-056 | `cargo clippy --workspace -- -D warnings` passes | — | auto | pending |

Regression tests are not individually numbered beyond the above.
The following MUST pass after all Phase 2.97 changes:

- `cargo test --workspace` — all existing Phase 1/2/2.5/2.7/2.8/2.9/2.95
  tests MUST continue to pass (except tests superseded by IUX-006
  which are replaced, not deleted silently).
- For any batch that touches `cfg(windows)` code, `cargo check
  --workspace --all-targets --target x86_64-pc-windows-msvc` MUST
  pass when that target is installed.
- All `--json` output contracts MUST remain unchanged (except
  additive fields documented in `SPEC_CACHE_CLEAN.md`).
- Exit codes 0/1/2/3 MUST retain their semantics.
