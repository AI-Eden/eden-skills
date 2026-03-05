# SPEC_TABLE_FIX.md

Table rendering fix for visual alignment drift and width stability.

**Related contracts:**

- `spec/phase2.8/SPEC_TABLE_RENDERING.md` (table infrastructure — **overridden** by this spec)
- `spec/phase2.8/SPEC_OUTPUT_UPGRADE.md` (command output)

## 1. Problem Statement

Phase 2.8/2.9 table rendering in TTY mode used width strategies tied to
terminal size (`Dynamic` first, then `DynamicFullWidth`). This can cause
layout instability when terminal width changes:

- Very wide terminals produce oversized tables that are hard to scan.
- Column geometry depends on terminal width rather than content width.
- Some terminals show separator/header drift when table text includes
  ANSI styling attributes.

This affects every table in the CLI: `list`, `doctor`, `plan`,
`update`, `install --dry-run`, and `install --list`.

## 2. Root Cause

`ContentArrangement::DynamicFullWidth` always consumes all available
terminal width. This avoids right-side gaps, but it makes table shape
depend on terminal size and can over-stretch narrow content.

`ContentArrangement::Disabled` uses content-driven sizing: each column
is measured from header + cell content and rendered near its natural
width (with preset padding). This is stable and deterministic across
different terminal widths.

## 3. Fix: Content-Driven TTY Layout + Plain Table Text

### 3.1 Arrangement Change

The `UiContext::table()` factory method MUST use
`ContentArrangement::Disabled` for TTY output.

**Before (Phase 2.9 previous):**

```rust
table.set_content_arrangement(ContentArrangement::DynamicFullWidth);
```

**After (Phase 2.9):**

```rust
if human_tty {
    table.set_content_arrangement(ContentArrangement::Disabled);
} else {
    table.set_content_arrangement(ContentArrangement::Dynamic);
}
```

Non-TTY output (piped, CI) continues to use `Dynamic` with
`set_width(80)`, preserving bounded width behavior for pipelines/logs.

### 3.1.1 Table Text Style Rule

All table headers and table cell values MUST be rendered as plain text
without ANSI styling attributes. This applies regardless of `--color`
mode.

Examples of forbidden styling in table content:

- bold (`.bold()`)
- color (`.green()`, `.cyan()`, `.yellow()`, `.red()`)
- dim/italic/underline (`.dimmed()`, etc.)

This rule applies only to table header/cell content. Non-table output
lines (action prefixes, status symbols, warnings, summaries) may still
use styling according to existing output specs.

### 3.2 Column Constraint Policy

With content-driven sizing, columns naturally fit their content. Fixed
semantic columns (e.g., `Sev`, `Mode`, `#`) still MUST keep
`UpperBoundary` constraints to avoid accidental expansion from outlier
values and to preserve visual consistency.

Policy:

| Column Type | Constraint | Example Columns |
| :--- | :--- | :--- |
| **Fixed-width** | `UpperBoundary(Width::Fixed(N))` | `Sev` (5), `#` (4), `Mode` (8), `Status` (8) |
| **Elastic** | No constraint (absorbs surplus) | `Skill`, `Agents`, `Description`, `Detail` |

The `UiContext::table()` method itself does NOT apply constraints.
Constraints are applied at each call site after table creation:

```rust
let mut table = ui.table(&["Sev", "Code", "Skill"]);
if let Some(col) = table.column_mut(0) {
    col.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(5)));
}
```

### 3.3 Per-Table Constraint Definitions

| Table | Col 0 | Col 1 | Col 2 | Col 3 |
| :--- | :--- | :--- | :--- | :--- |
| `list` (Skill, Mode, Source, Agents) | elastic | UB(8) | elastic | elastic |
| `doctor` (Sev, Code, Skill) | UB(5) | elastic | elastic | — |
| `plan` (Action, Skill, Target, Mode) | UB(10) | elastic | elastic | UB(8) |
| `update` (Registry, Status, Detail) | elastic | UB(10) | elastic | — |
| `install --dry-run` (Agent, Path, Mode) | elastic | elastic | UB(8) | — |
| `install --list` (#, Name, Description) | UB(4) | elastic | elastic | — |
| `remove` candidates (#, Skill, Source) | UB(4) | elastic | elastic | — |
| `update` skills (Skill, Status) | elastic | UB(12) | — | — |

`UB(N)` = `ColumnConstraint::UpperBoundary(Width::Fixed(N))`.

## 4. Preset Unchanged

The border preset remains `UTF8_FULL_CONDENSED` for TTY and
`ASCII_FULL_CONDENSED` for non-TTY. No preset change is needed;
only the content-arrangement strategy changes.

## 5. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **TFX-001** | Builder | **P0** | TTY tables MUST use content-driven layout (`ContentArrangement::Disabled`), and table headers/cells MUST remain plain text (no ANSI styling attributes). | `UiContext::table()` sets `Disabled` when `human_tty` is true and does not style table headers/cells. |
| **TFX-002** | Builder | **P0** | Fixed-semantics columns MUST have `UpperBoundary` constraints per Section 3.3. | Each table call site applies constraints. |
| **TFX-003** | Builder | **P0** | Non-TTY tables MUST continue using `Dynamic` with `set_width(80)`. | Non-TTY output unchanged from Phase 2.8. |

## 6. Backward Compatibility

| Existing Feature | Phase 2.9 Behavior |
| :--- | :--- |
| `--json` output | Unchanged. Tables never appear in JSON mode. |
| Non-TTY piped output | Unchanged. `Dynamic` + width 80 + ASCII borders. |
| `NO_COLOR` / `FORCE_COLOR` / `CI` | Unchanged for non-table output. Table headers/cells remain plain text in all modes. |
| `--color` flag | Unchanged. |
