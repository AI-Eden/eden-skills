# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.9.
Use this file to recover accurate context after compression.

**Status:** DRAFT — populated with requirement IDs. Implementation
and test columns will be filled during Builder execution.

## 1. Table Fix Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| TFX-001 | `SPEC_TABLE_FIX.md` 3.1 | TTY tables MUST use `DynamicFullWidth` | | TM-P29-001, TM-P29-004, TM-P29-005 | pending |
| TFX-002 | `SPEC_TABLE_FIX.md` 3.2–3.3 | Fixed-width columns MUST have `UpperBoundary` constraints | | TM-P29-003 | pending |
| TFX-003 | `SPEC_TABLE_FIX.md` 3.1 | Non-TTY tables MUST use `Dynamic` + width 80 | | TM-P29-002 | pending |

## 2. Update Extension Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| UPD-001 | `SPEC_UPDATE_EXT.md` 2.1–2.2 | `update` MUST refresh Mode A skill sources | | TM-P29-006, TM-P29-013 | pending |
| UPD-002 | `SPEC_UPDATE_EXT.md` 2.2 | `update` without `--apply` MUST NOT mutate local state | | TM-P29-007 | pending |
| UPD-003 | `SPEC_UPDATE_EXT.md` 2.3 | `update --apply` MUST reconcile changed skills | | TM-P29-008 | pending |
| UPD-004 | `SPEC_UPDATE_EXT.md` 3.1 | Skill refresh results MUST render as table | | TM-P29-010 | pending |
| UPD-005 | `SPEC_UPDATE_EXT.md` 3.5 | Status values MUST be colored per palette | | TM-P29-011 | pending |
| UPD-006 | `SPEC_UPDATE_EXT.md` 3.3 | No registries + no skills: install guidance | | TM-P29-009 | pending |
| UPD-007 | `SPEC_UPDATE_EXT.md` 3.6 | `--json` MUST include `skills` array | | TM-P29-012 | pending |
| UPD-008 | `SPEC_UPDATE_EXT.md` 4 | Skill refresh MUST use reactor concurrency | | TM-P29-014 | pending |

## 3. Install UX Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| IUX-001 | `SPEC_INSTALL_UX.md` 2.2 | Discovery preview MUST use card-style numbered list | | TM-P29-015, TM-P29-019 | pending |
| IUX-002 | `SPEC_INSTALL_UX.md` 2.1 | Merge two discovery functions into one | | TM-P29-016 | pending |
| IUX-003 | `SPEC_INSTALL_UX.md` 2.2 | Descriptions dimmed and indented | | TM-P29-017, TM-P29-018 | pending |
| IUX-004 | `SPEC_INSTALL_UX.md` 3.2 | Step-style progress `[pos/len]` in TTY | | TM-P29-020 | pending |
| IUX-005 | `SPEC_INSTALL_UX.md` 3.3 | Styled sync summary after completion | | TM-P29-021, TM-P29-022 | pending |
| IUX-006 | `SPEC_INSTALL_UX.md` 4.1–4.3 | Tree-style grouped install results | | TM-P29-023, TM-P29-024 | pending |
| IUX-007 | `SPEC_INSTALL_UX.md` 4.4 | Tree coloring: cyan paths, dimmed connectors | | TM-P29-025 | pending |
| IUX-008 | `SPEC_INSTALL_UX.md` 4.7 | `apply`/`repair` use tree-style display | | TM-P29-026 | pending |

## 4. Output Consistency Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| OCN-001 | `SPEC_OUTPUT_CONSISTENCY.md` 3.1 | `add` shows `✓ Added` | | TM-P29-028 | pending |
| OCN-002 | `SPEC_OUTPUT_CONSISTENCY.md` 3.2 | `set` shows `✓ Updated` | | TM-P29-029 | pending |
| OCN-003 | `SPEC_OUTPUT_CONSISTENCY.md` 3.3 | `config import` shows `✓ Imported` | | TM-P29-030 | pending |
| OCN-004 | `SPEC_OUTPUT_CONSISTENCY.md` 3.4–3.8 | All warnings through `print_warning()` | | TM-P29-031 | pending |
| OCN-005 | `SPEC_OUTPUT_CONSISTENCY.md` 3.5 | `remove` cancel uses skipped symbol | | TM-P29-032 | pending |
| OCN-006 | `SPEC_OUTPUT_CONSISTENCY.md` 3.6 | `remove` candidates render as table | | TM-P29-033 | pending |
| OCN-007 | `SPEC_OUTPUT_CONSISTENCY.md` 4.1 | File paths styled cyan | | TM-P29-034 | pending |
| OCN-008 | `SPEC_OUTPUT_CONSISTENCY.md` 4.4 | Skill names bold in result lines | | TM-P29-034 | pending |
| OCN-009 | `SPEC_OUTPUT_CONSISTENCY.md` 4.4 | Mode labels and connectors dimmed | | TM-P29-034 | pending |
| OCN-010 | `SPEC_OUTPUT_CONSISTENCY.md` 4.2 | `UiContext::styled_path()` exists | | TM-P29-035 | pending |

## 5. Newline Policy Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| NLP-001 | `SPEC_NEWLINE_POLICY.md` 2.1 | No trailing blank line after output | | TM-P29-039, TM-P29-040 | pending |
| NLP-002 | `SPEC_NEWLINE_POLICY.md` 2.3 | Error: blank line only when hint exists | | TM-P29-036, TM-P29-037 | pending |
| NLP-003 | `SPEC_NEWLINE_POLICY.md` 3.2 | Clap errors `.trim_end()` | | TM-P29-038 | pending |
| NLP-004 | `SPEC_NEWLINE_POLICY.md` 2.2 | Section spacing per policy table | | TM-P29-039 | pending |
| NLP-005 | `SPEC_NEWLINE_POLICY.md` 3.4 | Full output-path audit | | TM-P29-040 | pending |
| NLP-006 | `SPEC_NEWLINE_POLICY.md` 3.4 | No trailing empty `println!()` before `Ok(())` | | TM-P29-040 | pending |
