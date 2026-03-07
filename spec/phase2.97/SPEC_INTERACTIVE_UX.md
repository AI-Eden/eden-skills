# SPEC_INTERACTIVE_UX.md

Interactive checkbox-based skill selection for both `remove` and
`install` commands.

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

## 2. Interactive Selector Component

### 2.1 Shared Infrastructure

Both `remove` and `install` MUST use a shared interactive checkbox
selector implementation with a custom renderer (`SkillSelectTheme`).

The `SkillSelectTheme` MUST:

1. Store a `HashMap<String, String>` mapping skill name to description.
2. Use checkbox characters consistent with the reference screenshots:
   `☐` (unchecked), `■` (checked).
3. Render unchecked items in gray text.
4. Render the currently active unchecked checkbox in **cyan**.
5. Render checked checkboxes in **green**.
6. Avoid bold styling for prompt items.
7. For `install`:
   - When the item is active and has a description, render
     `skill-name (description truncated...)`.
   - When the item is checked and has a description, keep rendering the
     description inline even after the cursor moves away.
   - When the item is neither active nor checked, render `skill-name`
     only.
8. For `remove`, never render descriptions inline.

### 2.2 Viewport Behavior

The selector MUST use a scrollable viewport derived from terminal
height.

- If items exist above the visible window, the top visible row MUST be
  `...`.
- If items exist below the visible window, the bottom visible row MUST
  be `...`.
- The renderer MUST avoid soft-wrapping prompt rows. If the formatted
  item would exceed terminal width, the description MUST be further
  shortened or suppressed so the row still fits on one terminal line.

### 2.3 Test Injection

For automated testing, the existing environment variable short-circuit
pattern MUST be preserved:

- `EDEN_SKILLS_TEST_REMOVE_INPUT`: comma-separated 0-based indices
  (e.g., `"0,2"` selects items 0 and 2), or `"interrupt"` to simulate
  Ctrl+C.
- `EDEN_SKILLS_TEST_SKILL_INPUT`: same format for install selection.
- When the env var is set, the interactive selector is bypassed and the specified
  indices are returned directly.

## 3. Remove Interactive Mode

### 3.1 Entry Condition

When `eden-skills remove` is invoked without skill ID arguments and
the terminal is interactive, the checkbox selector prompt MUST be shown.

### 3.2 Prompt

```text
◆ Select skills to remove (space to toggle)
   ◻api-design-principles
   ◻async-python-patterns
   ◻brand-guidelines
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
**removed** from interactive `remove`.

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
interactive, the checkbox selector prompt MUST be shown.

### 4.2 Prompt with Description

```text
◆ Select skills to install (space to toggle)
   ◻deploy-to-vercel (Deploy applications and websites to Vercel. Use when the ...)
   ◻vercel-composition-patterns
   ◻vercel-react-best-practices
   ◻vercel-react-native-skills
   ◻web-design-guidelines
```

The active item shows its description inline. Once an item is toggled
on, its inline description remains visible even after the cursor moves
away. The description is rendered in **dim** style with one space
separating the name and the opening parenthesis.

### 4.3 Description Truncation

```text
visible_description_chars = min(description_chars, 57)
```

If the description is longer than 57 visible characters, truncate to 57
characters and append `...` (the 57-character budget excludes the
surrounding parentheses). If terminal width is still insufficient after
57-character truncation, shorten further or suppress the description to
avoid soft-wrapping.

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
| `*` wildcard in remove | **Removed** — no special meaning |

## 6. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **IUX-001** | Builder | **P1** | `remove` interactive mode MUST use the shared checkbox selector. | Running `remove` without IDs shows a scrollable checkbox list with overflow indicators. |
| **IUX-002** | Builder | **P1** | `install` interactive mode MUST use the shared checkbox selector. | Running `install <repo>` with multiple skills shows a scrollable checkbox list with overflow indicators. |
| **IUX-003** | Builder | **P1** | `SkillSelectTheme` MUST show inline descriptions for active install items and keep them visible for checked install items. | Hovering or checking a skill with description shows `name (desc...)`. |
| **IUX-004** | Builder | **P1** | Description MUST be dim, capped at 57 characters before `...`, and rendered without soft-wrapping. | Long descriptions are truncated with `...` and remain on one terminal line. |
| **IUX-005** | Builder | **P1** | `remove` MUST show a `Confirm` prompt after MultiSelect selection. | After selection, `Remove N skills? (y/N)` is shown. |
| **IUX-006** | Builder | **P1** | `remove` `*` wildcard feature (`RMA-001~004`) MUST be removed. | Input `*` is no longer recognized as special syntax. |
| **IUX-007** | Builder | **P1** | Test env vars MUST bypass the interactive selector and return specified indices. | Tests using `EDEN_SKILLS_TEST_REMOVE_INPUT="0,2"` select correct items. |
| **IUX-008** | Builder | **P1** | Non-interactive fallback MUST preserve existing behavior. | Non-TTY remove requires explicit IDs; non-TTY install installs all. |
| **IUX-009** | Builder | **P1** | Active and checked states MUST be indicated by color without bold text. | Active unchecked checkbox is cyan; checked checkbox is green; prompt item text is not bold. |
| **IUX-010** | Builder | **P1** | Items without description MUST show name only (no empty parentheses). | Skills with empty description show just the name when hovered. |
