# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.97.
Use this file to recover accurate context after compression.

**Status:** PENDING — Populated by Builder during implementation.

## 1. Update Fix Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| UFX-001 | `SPEC_UPDATE_FIX.md` 2.1 | Deduplicate refresh tasks by repo_cache_key | `crates/eden-skills-cli/src/commands/update.rs`, `crates/eden-skills-cli/tests/update_fix_tests.rs` | TM-P297-001, TM-P297-005 | completed |
| UFX-002 | `SPEC_UPDATE_FIX.md` 2.2 | Broadcast refresh status to all grouped skills | `crates/eden-skills-cli/src/commands/update.rs`, `crates/eden-skills-cli/tests/update_fix_tests.rs` | TM-P297-002, TM-P297-006 | completed |
| UFX-003 | `SPEC_UPDATE_FIX.md` 2.3 | Clean stale .git lock files before fetch | `crates/eden-skills-cli/src/commands/update.rs`, `crates/eden-skills-cli/tests/update_fix_tests.rs` | TM-P297-003, TM-P297-004 | completed |

## 2. Table Style Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| TST-001 | `SPEC_TABLE_STYLE.md` 2.1 | Enable custom_styling feature | `crates/eden-skills-cli/Cargo.toml` | TM-P297-007 | completed |
| TST-002 | `SPEC_TABLE_STYLE.md` 3.1 | Bold table headers | `crates/eden-skills-cli/src/ui.rs` | TM-P297-008, TM-P297-012 | completed |
| TST-003 | `SPEC_TABLE_STYLE.md` 3.2 | Bold+magenta Skill ID cells | `crates/eden-skills-cli/src/ui.rs`, `crates/eden-skills-cli/src/commands/config_ops.rs`, `crates/eden-skills-cli/src/commands/update.rs`, `crates/eden-skills-cli/src/commands/diagnose.rs`, `crates/eden-skills-cli/src/commands/plan_cmd.rs`, `crates/eden-skills-cli/src/commands/install.rs` | TM-P297-009 | completed |
| TST-004 | `SPEC_TABLE_STYLE.md` 3.3 | Status cells colored by category | `crates/eden-skills-cli/src/ui.rs`, `crates/eden-skills-cli/src/commands/update.rs` | TM-P297-010 | completed |
| TST-005 | `SPEC_TABLE_STYLE.md` 3 | Styled cells do not break alignment | `crates/eden-skills-cli/Cargo.toml`, `crates/eden-skills-cli/src/ui.rs` | TM-P297-011 | completed |
| TST-006 | `SPEC_TABLE_STYLE.md` 5 | clap help colorization | `crates/eden-skills-cli/src/lib.rs` | TM-P297-057 | completed |
| TST-007 | `SPEC_TABLE_STYLE.md` 6.1 | List table Path column | `crates/eden-skills-cli/src/commands/config_ops.rs` | TM-P297-058 | completed |
| TST-008 | `SPEC_TABLE_STYLE.md` 6.2 | List Agents truncation | `crates/eden-skills-cli/src/commands/config_ops.rs` | TM-P297-059 | completed |

## 3. Interactive UX Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| IUX-001 | `SPEC_INTERACTIVE_UX.md` 3 | Remove uses MultiSelect | — | TM-P297-013, TM-P297-014 | pending |
| IUX-002 | `SPEC_INTERACTIVE_UX.md` 4 | Install uses MultiSelect | — | TM-P297-021 | pending |
| IUX-003 | `SPEC_INTERACTIVE_UX.md` 4.2 | Active item shows inline description | — | TM-P297-022 | pending |
| IUX-004 | `SPEC_INTERACTIVE_UX.md` 4.3 | Description truncated to terminal width | — | TM-P297-023 | pending |
| IUX-005 | `SPEC_INTERACTIVE_UX.md` 3.3 | Confirm prompt after MultiSelect | — | TM-P297-014, TM-P297-015 | pending |
| IUX-006 | `SPEC_INTERACTIVE_UX.md` 3.4 | Remove `*` wildcard feature | — | TM-P297-019 | pending |
| IUX-007 | `SPEC_INTERACTIVE_UX.md` 2.3 | Test env var injection | — | TM-P297-016, TM-P297-017, TM-P297-025 | pending |
| IUX-008 | `SPEC_INTERACTIVE_UX.md` 3.5, 4.5 | Non-interactive fallback | — | TM-P297-018, TM-P297-020, TM-P297-026, TM-P297-027, TM-P297-028 | pending |
| IUX-009 | `SPEC_INTERACTIVE_UX.md` 2.1 | Active item rendered bold | — | TM-P297-022 | pending |
| IUX-010 | `SPEC_INTERACTIVE_UX.md` 2.1 | No empty parentheses for description-less skills | — | TM-P297-024 | pending |

## 4. Cache Clean Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| CCL-001 | `SPEC_CACHE_CLEAN.md` 2.2 | Identify and remove orphaned .repos/ entries | — | TM-P297-029, TM-P297-033 | pending |
| CCL-002 | `SPEC_CACHE_CLEAN.md` 2.2 | Remove stale discovery temp dirs | — | TM-P297-030 | pending |
| CCL-003 | `SPEC_CACHE_CLEAN.md` 2.2 | Dry-run lists without deleting | — | TM-P297-031 | pending |
| CCL-004 | `SPEC_CACHE_CLEAN.md` 2.4 | JSON output schema | — | TM-P297-032 | pending |
| CCL-005 | `SPEC_CACHE_CLEAN.md` 3 | remove --auto-clean runs clean after removal | — | TM-P297-034 | pending |
| CCL-006 | `SPEC_CACHE_CLEAN.md` 4 | Doctor reports ORPHAN_CACHE_ENTRY | — | TM-P297-035 | pending |
| CCL-007 | `SPEC_CACHE_CLEAN.md` 2.3 | Report freed disk space | — | TM-P297-036 | pending |

## 5. Docker Managed Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| DMG-001 | `SPEC_DOCKER_MANAGED.md` 3 | Install writes .eden-managed entry | — | TM-P297-037, TM-P297-038 | pending |
| DMG-002 | `SPEC_DOCKER_MANAGED.md` 3.1 | Docker install sets source: "external" | — | TM-P297-037 | pending |
| DMG-003 | `SPEC_DOCKER_MANAGED.md` 3.2 | Local install sets source: "local" | — | TM-P297-038 | pending |
| DMG-004 | `SPEC_DOCKER_MANAGED.md` 4.1 | Remove guard for external skills | — | TM-P297-039, TM-P297-046 | pending |
| DMG-005 | `SPEC_DOCKER_MANAGED.md` 4.1 | --force overrides remove guard | — | TM-P297-040 | pending |
| DMG-006 | `SPEC_DOCKER_MANAGED.md` 4.2 | Install guard for external skills | — | TM-P297-041 | pending |
| DMG-007 | `SPEC_DOCKER_MANAGED.md` 5 | Doctor ownership findings | — | TM-P297-042, TM-P297-043 | pending |
| DMG-008 | `SPEC_DOCKER_MANAGED.md` 6.3–6.4 | Missing/corrupted manifest tolerance | — | TM-P297-044, TM-P297-045 | pending |

## 6. Hint Sync Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| HSY-001 | `SPEC_HINT_SYNC.md` 2.1 | All hints use `~>` prefix | `crates/eden-skills-cli/src/main.rs`, `crates/eden-skills-cli/src/commands/update.rs`, `crates/eden-skills-cli/src/commands/diagnose.rs`, `crates/eden-skills-cli/src/commands/plan_cmd.rs`, `crates/eden-skills-cli/src/commands/install.rs` | TM-P297-047, TM-P297-049, TM-P297-050 | completed |
| HSY-002 | `SPEC_HINT_SYNC.md` 2.2 | `~>` styled magenta | `crates/eden-skills-cli/src/ui.rs`, `crates/eden-skills-cli/src/main.rs`, `crates/eden-skills-cli/src/commands/update.rs`, `crates/eden-skills-cli/src/commands/diagnose.rs`, `crates/eden-skills-cli/src/commands/plan_cmd.rs`, `crates/eden-skills-cli/src/commands/install.rs` | TM-P297-048 | completed |

## 7. Documentation Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| DOC-001 | `README.md` | README updated with new commands and flags | — | TM-P297-051, TM-P297-052, TM-P297-054 | pending |
| DOC-002 | `docs/` | User docs updated with interactive UX and new features | — | TM-P297-053 | pending |
