# SPEC_TEST_MATRIX.md

Phase 2.9 acceptance test scenarios.

## 1. Convention

- Scenario IDs: `TM-P29-001` to `TM-P29-043`.
- Tests marked `auto` are implemented as Rust integration tests.
- Tests marked `manual` require manual verification (e.g., visual
  inspection of colored output in a real terminal).

## 2. Table Fix (WP-1)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P29-001 | TTY tables use content-driven layout (`Disabled`) and keep header/cell text plain (no ANSI styling attributes) | TFX-001 | auto | pending |
| TM-P29-002 | Non-TTY tables use `Dynamic` with width 80 | TFX-003 | auto | pending |
| TM-P29-003 | Fixed-width columns respect `UpperBoundary` constraints | TFX-002 | auto | pending |
| TM-P29-004 | `list` table remains content-width on wide terminal (not full-width stretched) | TFX-001 | manual | completed |
| TM-P29-005 | `doctor` table remains content-width and visually aligned | TFX-001 | manual | completed |

## 3. Update Extension (WP-2)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P29-006 | `update` with Mode A skills fetches and reports status | UPD-001 | auto | pending |
| TM-P29-007 | `update` without `--apply` does not mutate local state | UPD-002 | auto | pending |
| TM-P29-008 | `update --apply` reconciles skills with new commits | UPD-003 | auto | pending |
| TM-P29-009 | `update` with no registries and no skills shows guidance | UPD-006 | auto | pending |
| TM-P29-010 | `update` skill refresh renders as table | UPD-004 | auto | pending |
| TM-P29-011 | `update` skill status cells are plain labels (no ANSI styling attributes) | UPD-005 | auto | pending |
| TM-P29-012 | `update --json` includes `skills` array in output | UPD-007 | auto | pending |
| TM-P29-013 | `update` with registries + skills shows both sections | UPD-001 | auto | pending |
| TM-P29-014 | `update` skill refresh uses reactor concurrency | UPD-008 | auto | pending |

## 4. Install UX (WP-3)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P29-015 | `install --list` shows card-style numbered list | IUX-001 | auto | pending |
| TM-P29-016 | Interactive install preview shows same card format as `--list` | IUX-002 | auto | pending |
| TM-P29-017 | Skills with description show dimmed text on indented line | IUX-003 | auto | pending |
| TM-P29-018 | Skills without description show name-only line | IUX-003 | auto | pending |
| TM-P29-019 | Discovery preview truncates at 8 skills (non-`--list` mode) | IUX-001 | auto | pending |
| TM-P29-020 | Source sync shows step-style `[pos/len]` progress in TTY | IUX-004 | manual | completed |
| TM-P29-021 | Source sync shows summary line after completion | IUX-005 | auto | pending |
| TM-P29-022 | Non-TTY source sync skips progress bar, shows summary only | IUX-005 | auto | pending |
| TM-P29-023 | Install results use tree display with `├─` and `└─` | IUX-006 | auto | pending |
| TM-P29-024 | Tree groups skills — name appears once per group | IUX-006 | auto | pending |
| TM-P29-025 | Tree paths are cyan, connectors dimmed, mode dimmed | IUX-007 | manual | completed |
| TM-P29-026 | `apply`/`repair` use tree-style install lines | IUX-008 | auto | pending |
| TM-P29-027 | `install --list --json` output unchanged | IUX-001 | auto | pending |
| TM-P29-041 | `install --dry-run` (multi-skill) renders titled table sections for skill preview and targets | IUX-009 | auto | pending |
| TM-P29-042 | `install --dry-run` skill table defaults to first 8 rows and shows truncation footer | IUX-009 | auto | pending |
| TM-P29-043 | `install --dry-run --list` shows all selected skill rows (no truncation footer) | IUX-009 | auto | pending |

## 5. Output Consistency (WP-4)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P29-028 | `add` shows `✓ Added 'id'` with abbreviated path | OCN-001 | auto | pending |
| TM-P29-029 | `set` shows `✓ Updated 'id'` with abbreviated path | OCN-002 | auto | pending |
| TM-P29-030 | `config import` shows `✓ Imported config` with abbreviated path | OCN-003 | auto | pending |
| TM-P29-031 | No raw `eprintln!("warning:")` remains in codebase | OCN-004 | auto | pending |
| TM-P29-032 | `remove` cancellation shows `· Remove cancelled` | OCN-005 | auto | pending |
| TM-P29-033 | `remove` interactive candidates render as table | OCN-006 | auto | pending |
| TM-P29-034 | File paths displayed in cyan when colors enabled | OCN-007 | manual | completed |
| TM-P29-035 | `UiContext::styled_path()` method exists | OCN-010 | auto | pending |

## 6. Newline Policy (WP-5)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P29-036 | Error without hint has no trailing blank line | NLP-002 | auto | pending |
| TM-P29-037 | Error with hint has exactly one blank line between error and hint | NLP-002 | auto | pending |
| TM-P29-038 | Clap error (e.g., `eden-skills lis`) has no trailing blank lines | NLP-003 | auto | pending |
| TM-P29-039 | `list`, `doctor`, `plan` end without trailing blank line | NLP-001 | auto | pending |
| TM-P29-040 | `install`, `remove`, `update` end without trailing blank line | NLP-001 | auto | pending |

## 7. Regression

Regression tests are not individually numbered. The following MUST
pass after all Phase 2.9 changes:

- `cargo test --workspace` — all existing Phase 1/2/2.5/2.7/2.8
  tests MUST continue to pass.
- All `--json` output contracts MUST remain unchanged.
- Exit codes 0/1/2/3 MUST retain their semantics.
- `rg '\x1b\[' crates/` — zero hardcoded ANSI sequences in source.
