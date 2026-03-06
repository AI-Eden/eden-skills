# SPEC_TABLE_STYLE.md

Table content styling via `comfy-table` `custom_styling` feature.

**Related contracts:**

- `spec/phase2.8/SPEC_TABLE_RENDERING.md` (table infrastructure)
- `spec/phase2.9/SPEC_TABLE_FIX.md` (content-driven sizing)

## 1. Problem Statement

Table cell content currently has no ANSI styling. Adding styles (bold,
color) to cell content breaks column alignment because `comfy-table`
counts ANSI escape bytes as visible characters. The `custom_styling`
feature flag solves this by stripping ANSI codes during width
measurement.

## 2. Feature Flag

### 2.1 Cargo.toml Change

`crates/eden-skills-cli/Cargo.toml` MUST change:

```toml
# Before
comfy-table = "7"

# After
comfy-table = { version = "7", features = ["custom_styling"] }
```

### 2.2 Performance Impact

The `custom_styling` feature adds 30-50% overhead to table rendering
(per upstream documentation). Absolute impact is negligible for CLI
output (~30 µs → ~45 µs).

## 3. Styling Rules

All styling rules apply only when `UiContext::colors_enabled()` is
true. In non-TTY / `--color never` mode, all cells are plain text.

### 3.1 Table Headers

All table headers MUST be rendered with **bold** attribute.

### 3.2 Skill ID Column

The Skill ID column (present in `list`, `update`, `remove`, `install
--dry-run`, `plan`, `doctor`) MUST be rendered with **bold + magenta**
foreground.

### 3.3 Status Column

Status values MUST be colored by semantic category:

| Status Value | Color |
| :--- | :--- |
| `up-to-date`, `ok`, `noop` | green |
| `failed`, `error` | red |
| `warning`, `conflict` | yellow |
| `skipped`, `missing` | dim (gray) |
| `cloned`, `updated`, `new commit` | cyan |

### 3.4 Source / Path Column

Source URLs and filesystem paths MUST use **cyan** foreground,
consistent with existing `ui.styled_path()` convention.

### 3.5 Mode / Detail Columns

Mode labels (`symlink`, `copy`) and secondary detail text MUST use
**dim** style.

## 4. Implementation Strategy

Table cell content MUST be styled by wrapping the string value with
`owo-colors` methods before inserting into the `comfy-table` row.
The `custom_styling` feature ensures `comfy-table` strips ANSI codes
during column width calculation.

```rust
// Example: styled skill ID cell
fn style_skill_id_cell(ui: &UiContext, id: &str) -> String {
    if ui.colors_enabled() {
        id.bold().magenta().to_string()
    } else {
        id.to_string()
    }
}
```

## 5. Backward Compatibility

| Existing Feature | Phase 2.97 Behavior |
| :--- | :--- |
| `--json` output | Unchanged — no ANSI in JSON |
| Non-TTY output | Unchanged — plain text cells |
| `--color never` | Unchanged — plain text cells |
| Table structure and columns | Unchanged |

## 6. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **TST-001** | Builder | **P0** | `comfy-table` MUST be configured with `custom_styling` feature. | `Cargo.toml` contains `features = ["custom_styling"]`. |
| **TST-002** | Builder | **P0** | Table headers MUST render bold when colors are enabled. | Visual inspection of `list`, `update`, `doctor` tables. |
| **TST-003** | Builder | **P0** | Skill ID cells MUST render bold+magenta when colors are enabled. | Skill ID column contains ANSI bold+magenta sequences. |
| **TST-004** | Builder | **P0** | Status cells MUST be colored per Section 3.3 semantic categories. | Status column contains appropriate color codes. |
| **TST-005** | Builder | **P1** | ANSI-styled cells MUST NOT break column alignment. | Table with styled content has consistent column widths. |
