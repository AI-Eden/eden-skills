# SPEC_INTERACTIVE_UX.md

Interactive skill selection using `dialoguer::MultiSelect` for both
`remove` and `install` commands.

**Related contracts:**

- `spec/phase2.7/SPEC_REMOVE_ENH.md` (batch remove, interactive mode)
- `spec/phase2.95/SPEC_REMOVE_ALL.md` (wildcard — **superseded**)
- `spec/phase2.5/SPEC_INSTALL_URL.md` (install discovery + selection)

## 1. Problem Statement

1. **`remove` interactive mode** lists all skills in a table and
   requires manual text input of numbers or names. No visual selection
   feedback, no scrolling viewport, poor UX for large skill sets.
2. **`install` interactive mode** asks "Install all N skills?" then
   requires manual text input of names. Same issues as remove.
3. Both interactions lack the modern checkbox-list UX pattern that
   competing tools (e.g., `npx skills`) provide.

## 2. MultiSelect Component

### 2.1 Shared Infrastructure

Both `remove` and `install` MUST use `dialoguer::MultiSelect` with a
custom `Theme` implementation (`SkillSelectTheme`).

The `SkillSelectTheme` MUST:

1. Wrap `dialoguer::theme::ColorfulTheme` as the base theme.
2. Store a `HashMap<String, String>` mapping skill name to description.
3. Override `format_multi_select_prompt_item`:
   - When `active == true` and a description exists: render
     `☐/☑ skill-name (description truncated...)` with the description
     portion in **dim** style, truncated to fit terminal width.
   - When `active == false` or no description: render `☐/☑ skill-name`
     only.
4. Use checkbox characters consistent with the reference screenshots:
   `☐` (unchecked), `☑` (checked), with the current/active item in
   **bold**.

### 2.2 Viewport Behavior

`dialoguer::MultiSelect` provides built-in scrollable viewport that
adapts to terminal height. No explicit page size configuration is
required for the default behavior. The `...` overflow indicators
are rendered automatically by `dialoguer` when items exceed the
visible area.

### 2.3 Test Injection

For automated testing, the existing environment variable short-circuit
pattern MUST be preserved:

- `EDEN_SKILLS_TEST_REMOVE_INPUT`: comma-separated 0-based indices
  (e.g., `"0,2"` selects items 0 and 2), or `"interrupt"` to simulate
  Ctrl+C.
- `EDEN_SKILLS_TEST_SKILL_INPUT`: same format for install selection.
- When the env var is set, `MultiSelect` is bypassed and the specified
  indices are returned directly.

## 3. Remove Interactive Mode

### 3.1 Entry Condition

When `eden-skills remove` is invoked without skill ID arguments and
the terminal is interactive, the MultiSelect prompt MUST be shown.

### 3.2 Prompt

```text
◆ Select skills to remove (space to toggle)
  ☐ api-design-principles
  ☐ async-python-patterns
  ☐ brand-guidelines
  ...
```

### 3.3 Confirmation

After selection, a `Confirm` prompt MUST be shown:

```text
Remove 3 skills? (y/N)
```

Default is `N` (reject). This preserves the existing safety behavior.

### 3.4 Wildcard Removal

The Phase 2.95 `*` wildcard feature (`RMA-001` through `RMA-004`) is
**superseded** by MultiSelect's native toggle-all capability.

- The `parse_remove_selection` function and its `*` handling MUST be
  removed.
- The `remove_selection_prompt` text-input flow MUST be removed.
- The `print_remove_candidates` table MUST be removed.
- Existing tests for `*` wildcard (`TM-P295-010` through `TM-P295-015`)
  are replaced by new MultiSelect tests.

### 3.5 Non-Interactive Fallback

When `!ui.interactive_enabled()` (non-TTY or `--json`), the existing
behavior is preserved: require explicit skill IDs on the command line.

## 4. Install Interactive Mode

### 4.1 Entry Condition

When `eden-skills install <source>` discovers multiple skills and
neither `--all` nor `--skill` is specified, and the terminal is
interactive, the MultiSelect prompt MUST be shown.

### 4.2 Prompt with Description

```text
◆ Select skills to install (space to toggle)
  ☐ deploy-to-vercel (Deploy applications and websites to Vercel. Use when the ...)
  ☐ vercel-composition-patterns
  ☐ vercel-react-best-practices
  ☐ vercel-react-native-skills
  ☐ web-design-guidelines
```

Only the **active** (currently hovered) item shows its description
inline. The description is rendered in **dim** style with one space
separating the name and the opening parenthesis. The description is
truncated with `...` to fit within the terminal width.

### 4.3 Description Truncation

```text
available_width = terminal_width - indent - checkbox - name_len - 2
```

Where `2` accounts for the space and opening parenthesis. If
`available_width < 10`, the description is suppressed entirely.
If the description fits within `available_width`, it is shown in
full (without truncation or trailing `...`).

### 4.4 Existing Flows Preserved

| Flag | Behavior |
| :--- | :--- |
| `--all` / `-y` | Install all discovered skills, no prompt |
| `--skill <name>` | Install named skills, no prompt |
| `--list` | List-only mode, no prompt |
| `--dry-run` | Preview mode, no prompt |
| Single skill discovered | Install directly, no prompt |

### 4.5 Non-Interactive Fallback

When `!ui.interactive_enabled()`, all discovered skills are installed
(same as `--all`).

## 5. Backward Compatibility

| Existing Feature | Phase 2.97 Behavior |
| :--- | :--- |
| `remove skill-a skill-b` (explicit IDs) | Unchanged |
| `remove -y` | Unchanged — skips interactive |
| `install --all` / `--skill` | Unchanged |
| `install --list` / `--dry-run` | Unchanged |
| `--json` mode | Unchanged — no interactive prompts |
| `*` wildcard in remove | **Removed** — superseded by MultiSelect |

## 6. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **IUX-001** | Builder | **P1** | `remove` interactive mode MUST use `MultiSelect` with checkbox selection. | Running `remove` without IDs shows scrollable checkbox list. |
| **IUX-002** | Builder | **P1** | `install` interactive mode MUST use `MultiSelect` with checkbox selection. | Running `install <repo>` with multiple skills shows checkbox list. |
| **IUX-003** | Builder | **P1** | `SkillSelectTheme` MUST show description inline for the active item only. | Hovering over a skill with description shows `name (desc...)`. |
| **IUX-004** | Builder | **P1** | Description MUST be rendered in dim style and truncated to terminal width. | Long descriptions are truncated with `...`. |
| **IUX-005** | Builder | **P1** | `remove` MUST show a `Confirm` prompt after MultiSelect selection. | After selection, `Remove N skills? (y/N)` is shown. |
| **IUX-006** | Builder | **P1** | `remove` `*` wildcard feature (`RMA-001~004`) MUST be removed. | Input `*` is no longer recognized as special syntax. |
| **IUX-007** | Builder | **P1** | Test env vars MUST bypass MultiSelect and return specified indices. | Tests using `EDEN_SKILLS_TEST_REMOVE_INPUT="0,2"` select correct items. |
| **IUX-008** | Builder | **P1** | Non-interactive fallback MUST preserve existing behavior. | Non-TTY remove requires explicit IDs; non-TTY install installs all. |
| **IUX-009** | Builder | **P1** | Active item in MultiSelect MUST be rendered bold. | Currently hovered item label is bold. |
| **IUX-010** | Builder | **P1** | Items without description MUST show name only (no empty parentheses). | Skills with empty description show just the name when hovered. |
