# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.98.
Use this file to recover accurate context after compression.

**Status:** IN PROGRESS — Batch 1 implementation mapped; documentation row pending.

## 1. List Source Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| LSR-001 | `SPEC_LIST_SOURCE.md` 3 | `list` table shows `Source` header | `crates/eden-skills-cli/src/commands/config_ops.rs` | TM-P298-001 | completed |
| LSR-002 | `SPEC_LIST_SOURCE.md` 2 | Source column renders `owner/repo (subpath)` | `crates/eden-skills-cli/src/commands/config_ops.rs` | TM-P298-002, TM-P298-003, TM-P298-006 | completed |
| LSR-003 | `SPEC_LIST_SOURCE.md` 2.3 | Source column uses cyan styling | `crates/eden-skills-cli/src/commands/config_ops.rs` | TM-P298-004, TM-P298-005 | completed |

## 2. Doctor UX Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| DUX-001 | `SPEC_DOCTOR_UX.md` 2.1 | `doctor` accepts `--no-warning` flag | `crates/eden-skills-cli/src/lib.rs`, `crates/eden-skills-cli/src/commands/diagnose.rs` | TM-P298-007 | completed |
| DUX-002 | `SPEC_DOCTOR_UX.md` 2.3 | `--no-warning` filters warning findings | `crates/eden-skills-cli/src/commands/diagnose.rs` | TM-P298-008, TM-P298-009 | completed |
| DUX-003 | `SPEC_DOCTOR_UX.md` 2.4 | `--no-warning` + `--strict` interaction | `crates/eden-skills-cli/src/commands/diagnose.rs` | TM-P298-010, TM-P298-011 | completed |
| DUX-004 | `SPEC_DOCTOR_UX.md` 3.1 | Summary table header `Sev` → `Level` | `crates/eden-skills-cli/src/commands/diagnose.rs` | TM-P298-012 | completed |
| DUX-005 | `SPEC_DOCTOR_UX.md` 3.2 | Cell value `warn` → `warning` | `crates/eden-skills-cli/src/commands/diagnose.rs` | TM-P298-013 | completed |
| DUX-006 | `SPEC_DOCTOR_UX.md` 4.1 | Level cell coloring | `crates/eden-skills-cli/src/commands/diagnose.rs` | TM-P298-014, TM-P298-015, TM-P298-016, TM-P298-017 | completed |

## 3. Verify Dedup Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| VDD-001 | `SPEC_VERIFY_DEDUP.md` 3.1 | Short-circuit checks when target missing | `crates/eden-skills-core/src/verify.rs` | TM-P298-018 | completed |
| VDD-002 | `SPEC_VERIFY_DEDUP.md` 3.3 | Existing targets run all checks normally | `crates/eden-skills-core/src/verify.rs` | TM-P298-019 | completed |
| VDD-003 | `SPEC_VERIFY_DEDUP.md` 4 | Repair works with reduced finding set | `crates/eden-skills-core/src/verify.rs`, `crates/eden-skills-cli/src/commands/reconcile.rs` | TM-P298-020 | completed |

## 4. Documentation Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| DOC-001 | `README.md`, `docs/` | Docs updated with new flag and column change | `README.md`, `docs/07-cli-reference.md` | — | pending |
