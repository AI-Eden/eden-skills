# SPEC_TEST_MATRIX.md

Phase 2.98 acceptance test scenarios.

## 1. Convention

- Scenario IDs: `TM-P298-001` to `TM-P298-020`.
- Tests marked `auto` are implemented as Rust integration tests.
- Tests marked `manual` require manual verification.

## 2. List Source (WP-1)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P298-001 | `list` table header shows `Source` instead of `Path` | LSR-001 | auto | pending |
| TM-P298-002 | `list` Source column renders `owner/repo (subpath)` for GitHub sources | LSR-002 | auto | pending |
| TM-P298-003 | `list` Source column renders `~/local-path (subpath)` for local sources | LSR-002 | auto | pending |
| TM-P298-004 | `list` Source column uses cyan ANSI styling in TTY mode | LSR-003 | auto | pending |
| TM-P298-005 | `list` Source column is plain text with `--color never` | LSR-003 | auto | pending |
| TM-P298-006 | `list --json` output schema is unchanged (source object preserved) | LSR-002 | auto | pending |

## 3. Doctor UX — `--no-warning` (WP-2)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P298-007 | `doctor --no-warning` is accepted without parse error | DUX-001 | auto | pending |
| TM-P298-008 | `doctor --no-warning` omits warning-severity findings from human output | DUX-002 | auto | pending |
| TM-P298-009 | `doctor --no-warning --json` omits warning findings from JSON array | DUX-002 | auto | pending |
| TM-P298-010 | `doctor --strict --no-warning` exits 0 when only warnings exist | DUX-003 | auto | pending |
| TM-P298-011 | `doctor --strict --no-warning` exits 3 when error findings exist | DUX-003 | auto | pending |

## 4. Doctor UX — Level Column (WP-2)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P298-012 | Doctor summary table header reads `Level` instead of `Sev` | DUX-004 | auto | pending |
| TM-P298-013 | Doctor summary table shows `warning` instead of `warn` | DUX-005 | auto | pending |
| TM-P298-014 | Doctor `Level` cell for `error` uses red ANSI in TTY mode | DUX-006 | auto | pending |
| TM-P298-015 | Doctor `Level` cell for `warning` uses yellow ANSI in TTY mode | DUX-006 | auto | pending |
| TM-P298-016 | Doctor `Level` cell for `info` uses dim ANSI in TTY mode | DUX-006 | auto | pending |
| TM-P298-017 | Doctor `Level` cell is plain text with `--color never` | DUX-006 | auto | pending |

## 5. Verify Dedup (WP-3)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P298-018 | Missing symlink target produces exactly 1 `TARGET_PATH_MISSING` finding (not 3) | VDD-001 | auto | pending |
| TM-P298-019 | Existing symlink with wrong target still produces `TARGET_RESOLVE_MISMATCH` | VDD-002 | auto | pending |
| TM-P298-020 | `repair` after `unlink` restores symlink correctly with single finding | VDD-003 | auto | pending |
