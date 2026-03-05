# SPEC_TABLE_FIX.md

Table rendering fix for the phantom empty column issue.

**Related contracts:**

- `spec/phase2.8/SPEC_TABLE_RENDERING.md` (table infrastructure — **overridden** by this spec)
- `spec/phase2.8/SPEC_OUTPUT_UPGRADE.md` (command output)

## 1. Problem Statement

Phase 2.8 introduced `comfy-table` with `ContentArrangement::Dynamic`
and the `UTF8_FULL_CONDENSED` preset. In TTY environments, when the
table's natural content width is less than the detected terminal width,
`Dynamic` mode leaves surplus space unallocated. The preset's right
border character still renders, producing a visible phantom column
(double `││` at the right edge).

This affects every table in the CLI: `list`, `doctor`, `plan`,
`update`, `install --dry-run`, and `install --list`.

## 2. Root Cause

`ContentArrangement::Dynamic` computes column widths to fit content
within the terminal width but does **not** distribute surplus space.
When the total content width plus borders is less than the terminal
width, the remaining space appears as an empty gap before the right
border, visually resembling an additional column.

`ContentArrangement::DynamicFullWidth` has the same fitting logic but
always distributes surplus space evenly across all columns, eliminating
the phantom gap.

## 3. Fix: TTY Content Arrangement

### 3.1 Arrangement Change

The `UiContext::table()` factory method MUST use
`ContentArrangement::DynamicFullWidth` for TTY output.

**Before (Phase 2.8):**

```rust
table.set_content_arrangement(ContentArrangement::Dynamic);
```

**After (Phase 2.9):**

```rust
if human_tty {
    table.set_content_arrangement(ContentArrangement::DynamicFullWidth);
} else {
    table.set_content_arrangement(ContentArrangement::Dynamic);
}
```

Non-TTY output (piped, CI) continues to use `Dynamic` with
`set_width(80)`, which is unaffected by the phantom column issue
because the width is explicitly bounded.

### 3.2 Column Constraint Policy

With `DynamicFullWidth`, surplus space is distributed evenly across
all columns. This can make narrow fixed-semantics columns (e.g.,
`Sev`, `Mode`, `#`) unnecessarily wide. To prevent this, each table
definition MUST apply `UpperBoundary` constraints on columns whose
content has a known maximum width.

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
the phantom column is eliminated by the arrangement mode change alone.

## 5. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **TFX-001** | Builder | **P0** | TTY tables MUST use `ContentArrangement::DynamicFullWidth`. | `UiContext::table()` sets `DynamicFullWidth` when `human_tty` is true. |
| **TFX-002** | Builder | **P0** | Fixed-semantics columns MUST have `UpperBoundary` constraints per Section 3.3. | Each table call site applies constraints. |
| **TFX-003** | Builder | **P0** | Non-TTY tables MUST continue using `Dynamic` with `set_width(80)`. | Non-TTY output unchanged from Phase 2.8. |

## 6. Backward Compatibility

| Existing Feature | Phase 2.9 Behavior |
| :--- | :--- |
| `--json` output | Unchanged. Tables never appear in JSON mode. |
| Non-TTY piped output | Unchanged. `Dynamic` + width 80 + ASCII borders. |
| `NO_COLOR` / `FORCE_COLOR` / `CI` | Unchanged. Table styling follows resolved color mode. |
| `--color` flag | Unchanged. |
