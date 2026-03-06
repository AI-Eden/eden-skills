# SPEC_REMOVE_ALL.md

Wildcard removal: `*` token in interactive remove selection.

**Related contracts:**

- `spec/phase2.7/SPEC_REMOVE_ENH.md` (batch remove, interactive selection)
- `spec/phase2.9/SPEC_OUTPUT_CONSISTENCY.md` (remove output formatting)

## 1. Problem Statement

When a user wants to remove all installed skills, the interactive
`remove` prompt requires entering every skill number or name
individually. There is no shortcut for "select all."

## 2. Wildcard Token

### 2.1 Recognition

The `parse_remove_selection()` function MUST recognize the `*` token
as a wildcard meaning "all configured skills."

When the user enters `*` (with optional surrounding whitespace), the
function MUST return all skill IDs from the config, in config order.

### 2.2 Mixed-Token Prohibition

`*` MUST NOT be combined with other tokens. If the input contains
`*` alongside numbers or names (e.g., `* 2 web-tool`), the function
MUST return an error:

```
error: '*' cannot be combined with other selections
```

### 2.3 Prompt Text Update

The interactive prompt MUST include a hint about the wildcard:

**Before:**

```
  Enter skill numbers or names to remove (space-separated):
```

**After:**

```
  Enter skill numbers or names to remove (space-separated, * for all):
```

## 3. Strengthened Confirmation

### 3.1 Trigger

When `*` is used (all skills selected), the confirmation prompt MUST
use a stronger wording than the standard removal confirmation.

### 3.2 Prompt Format

```
  ⚠ Remove ALL {N} skills? This cannot be undone. [y/N]
```

- The `⚠` symbol uses `StatusSymbol::Warning`.
- Default is `N` (same as standard removal confirmation).
- `--yes` / `-y` flag skips this confirmation (same as standard).

### 3.3 Implementation

A new function `confirm_remove_all()` (or a branch within
`confirm_remove_execution()`) MUST handle the strengthened prompt.
It follows the same interrupt-handling pattern as existing prompts
(`PromptInterruptGuard`, `take_prompt_interrupt()`).

## 4. UX Example

```text
  Skills   3 configured:

 ┌───┬──────────────────┬──────────────────────────┐
 │ # │ Skill            │ Source                   │
 ╞═══╪══════════════════╪══════════════════════════╡
 │ 1 │ browser-tool     │ vercel-labs/agent-skills │
 │ 2 │ code-review      │ user/code-review         │
 │ 3 │ filesystem-tool  │ vercel-labs/agent-skills │
 └───┴──────────────────┴──────────────────────────┘

  Enter skill numbers or names to remove (space-separated, * for all):
  > *

  ⚠ Remove ALL 3 skills? This cannot be undone. [y/N] y

  Remove  ✓ browser-tool
          ✓ code-review
          ✓ filesystem-tool

  ✓ 3 skills removed
```

## 5. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **RMA-001** | Builder | **P0** | `parse_remove_selection` MUST recognize `*` as a wildcard returning all skill IDs. | Input `*` returns full skill list. |
| **RMA-002** | Builder | **P0** | `*` combined with other tokens MUST produce an error. | Input `* 2` returns error message. |
| **RMA-003** | Builder | **P0** | Wildcard selection MUST trigger a strengthened confirmation with `⚠`, `[y/N]` default. | Prompt reads "Remove ALL N skills? This cannot be undone." |
| **RMA-004** | Builder | **P1** | Interactive prompt text MUST include `* for all` hint. | Prompt text updated. |

## 6. Backward Compatibility

| Existing Feature | Phase 2.95 Behavior |
| :--- | :--- |
| `remove <id> ...` (positional args) | Unchanged. `*` wildcard only applies to interactive prompt input. |
| `remove -y` / `remove --yes` | Unchanged. Skips confirmation for both normal and wildcard. |
| `remove --json` | Unchanged. |
| Non-TTY `remove` without args | Unchanged (still errors). |
