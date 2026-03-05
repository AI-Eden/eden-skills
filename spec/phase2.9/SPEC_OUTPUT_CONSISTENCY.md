# SPEC_OUTPUT_CONSISTENCY.md

Cross-command output consistency, path coloring, and remaining
UiContext gaps.

**Related contracts:**

- `spec/phase2.5/SPEC_CLI_UX.md` (visual design language)
- `spec/phase2.7/SPEC_OUTPUT_POLISH.md` (owo-colors, `--color` flag)
- `spec/phase2.8/SPEC_OUTPUT_UPGRADE.md` (UiContext unification)

## 1. Purpose

Phase 2.8 upgraded most commands to use `UiContext`, styled symbols,
and action prefixes. Several commands still emit raw unstyled output.
This spec closes the remaining gaps and introduces a path coloring
convention for visual hierarchy.

## 2. Audit of Remaining Gaps

| Command | File | Current Output | Issue |
| :--- | :--- | :--- | :--- |
| `add` | `config_ops.rs:274` | `println!("add: wrote {}", ...)` | Raw format, no symbol, no `~` abbreviation |
| `set` | `config_ops.rs:361` | `println!("set: wrote {}", ...)` | Raw format, no symbol, no `~` abbreviation |
| `config import` | `config_ops.rs:431` | `println!("config import: wrote {}", ...)` | Raw format, no symbol, no `~` abbreviation |
| `remove` | `remove.rs:74` | `eprintln!("warning: {warning}")` | Bypasses `print_warning()`; no UiContext |
| `remove` | `remove.rs:89` | `println!("remove cancelled.")` | No styled prefix, no symbol |
| `remove` | `remove.rs:250-262` | `print_remove_candidates(...)` | Manual alignment (`{:<16}`), not table; `source.repo` shown raw |
| `add` | `config_ops.rs:223` | `eprintln!("warning: {warning}")` | Bypasses `print_warning()` |
| `set` | `config_ops.rs:307` | `eprintln!("warning: {warning}")` | Bypasses `print_warning()` |
| `common.rs:507-510` | `validate_registry_manifest...` | `eprintln!("warning: ...")` | Bypasses `print_warning()`; no UiContext |

## 3. Command Output Upgrades

### 3.1 `add`

**Before:**

```text
add: wrote /home/eden/.eden-skills/skills.toml
```

**After:**

```text
  ✓ Added 'my-skill' to ~/.eden-skills/skills.toml
```

Implementation:

- Create `UiContext::from_env(options.json)` at function top.
- Use `status_symbol(Success)` + `abbreviate_home_path()`.
- Skill name in single quotes for clarity.

### 3.2 `set`

**Before:**

```text
set: wrote /home/eden/.eden-skills/skills.toml
```

**After:**

```text
  ✓ Updated 'my-skill' in ~/.eden-skills/skills.toml
```

Same pattern as `add`.

### 3.3 `config import`

**Before:**

```text
config import: wrote /home/eden/.eden-skills/skills.toml
```

**After:**

```text
  ✓ Imported config to ~/.eden-skills/skills.toml
```

### 3.4 `remove` — Warning Path

All `eprintln!("warning: {warning}")` calls in `remove.rs` MUST be
replaced with `print_warning(&ui, &warning)`.

### 3.5 `remove` — Cancellation

**Before:**

```text
remove cancelled.
```

**After:**

```text
  · Remove cancelled
```

The `·` (skipped symbol, dimmed) conveys "nothing happened" visually.

### 3.6 `remove` — Interactive Candidates

**Before:**

```text
  Skills   4 configured:

    1. vercel-composit (https://github.com/vercel-labs/agent-skills.git)
    2. vercel-react-be (https://github.com/vercel-labs/agent-skills.git)
```

**After:**

```text
  Skills   4 configured:

 ┌───┬──────────────────────────────┬──────────────────────────┐
 │ # │ Skill                        │ Source                   │
 ╞═══╪══════════════════════════════╪══════════════════════════╡
 │ 1 │ vercel-composition-patterns  │ vercel-labs/agent-skills │
 │ 2 │ vercel-react-best-practices  │ vercel-labs/agent-skills │
 │ 3 │ vercel-react-native-skills   │ vercel-labs/agent-skills │
 │ 4 │ web-design-guidelines        │ vercel-labs/agent-skills │
 └───┴──────────────────────────────┴──────────────────────────┘

  Enter skill numbers or names to remove (space-separated):
```

The table uses `ui.table()` with `#` (UB 4), `Skill` (elastic),
`Source` (elastic, `abbreviate_repo_url`). Column constraints per
`SPEC_TABLE_FIX.md` Section 3.3.

### 3.7 `add` / `set` — Warning Path

All `eprintln!("warning: {warning}")` calls in `config_ops.rs`
MUST be replaced with `print_warning(&ui, &warning)`.

### 3.8 Registry Manifest Warning

The `validate_registry_manifest_for_resolution()` function in
`common.rs` MUST accept a `&UiContext` parameter and use
`print_warning()` instead of raw `eprintln!`.

## 4. Path Coloring Convention

### 4.1 Policy

All **file system paths** displayed in human-mode output MUST be
styled with **cyan** when colors are enabled. This provides immediate
visual distinction between paths and surrounding text.

### 4.2 Implementation

Add a utility method to `UiContext`:

```rust
impl UiContext {
    pub fn styled_path(&self, path: &str) -> String {
        let abbreviated = abbreviate_home_path(path);
        if self.colors_enabled() {
            abbreviated.cyan().to_string()
        } else {
            abbreviated
        }
    }
}
```

This method combines `~` abbreviation and cyan coloring in one call.

### 4.3 Scope

`styled_path()` MUST be used in:

- Install tree target paths.
- `apply`/`repair` install lines.
- `init` success path.
- `add`/`set`/`config import` success path.
- `doctor` finding message paths.
- `plan` target paths.
- Error messages that contain file paths (via `abbreviate_message_paths`
  in `main.rs` — optionally extend to color if stderr supports it).

`styled_path()` MUST NOT be used in:

- `--json` output.
- Table cell content where `abbreviate_home_path()` is already applied
  (table cells handle their own styling).

### 4.4 Additional Element Styling

| Element | Style | Applied Where |
| :--- | :--- | :--- |
| Skill name in result lines | bold | Install tree, remove summary |
| Mode label `(symlink)` / `(copy)` | dimmed | Install tree, plan text lines |
| Tree connectors `├─`, `└─`, `│` | dimmed | Install tree |

## 5. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **OCN-001** | Builder | **P0** | `add` human output MUST show `✓ Added 'id'` with abbreviated path. | `add` output matches Section 3.1 format. |
| **OCN-002** | Builder | **P0** | `set` human output MUST show `✓ Updated 'id'` with abbreviated path. | `set` output matches Section 3.2 format. |
| **OCN-003** | Builder | **P0** | `config import` human output MUST show `✓ Imported config` with abbreviated path. | Output matches Section 3.3 format. |
| **OCN-004** | Builder | **P0** | All `eprintln!("warning: ...")` MUST go through `print_warning()`. | No raw `eprintln!("warning: ...")` in `remove.rs`, `config_ops.rs`, or `common.rs`. |
| **OCN-005** | Builder | **P0** | `remove` cancellation MUST use skipped symbol. | `· Remove cancelled` displayed on cancel. |
| **OCN-006** | Builder | **P0** | `remove` interactive candidates MUST render as table. | Table with `#`, `Skill`, `Source` columns. |
| **OCN-007** | Builder | **P0** | File system paths MUST be styled cyan in human mode via `styled_path()`. | Path elements appear in cyan when colors enabled. |
| **OCN-008** | Builder | **P0** | Skill names in result lines MUST be bold. | `owo_colors::bold()` applied to skill names. |
| **OCN-009** | Builder | **P1** | Mode labels and tree connectors MUST be dimmed. | `(symlink)`, `├─`, `└─` rendered in dimmed style. |
| **OCN-010** | Builder | **P1** | `UiContext` MUST expose `styled_path(&self, path: &str) -> String`. | Method exists and combines abbreviation + coloring. |

## 6. Backward Compatibility

| Existing Feature | Phase 2.9 Behavior |
| :--- | :--- |
| `add --json` | JSON output unchanged. |
| `set --json` | JSON output unchanged. |
| `remove --json` | JSON output unchanged. |
| `--color never` | All coloring disabled; paths are plain abbreviated text. |
| Non-TTY | No color; structure preserved. |
