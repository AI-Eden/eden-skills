# SPEC_INSTALL_UX.md

Install command UX overhaul: discovery preview, sync progress, and
result display.

**Related contracts:**

- `spec/phase2.5/SPEC_INSTALL_URL.md` (install flow)
- `spec/phase2.8/SPEC_OUTPUT_UPGRADE.md` Section 4.5–4.6 (install output — **superseded** by this spec)
- `spec/phase2.8/SPEC_TABLE_RENDERING.md` Section 5.3 (install --list table — **superseded** by this spec)

## 1. Purpose

Three aspects of the `install` command's human-mode output need
improvement:

1. **Discovery preview** — the current display uses two different
   functions (`print_discovered_skills` for `--list`,
   `print_discovery_summary` for interactive mode) with inconsistent
   formats. Long descriptions are poorly handled in table cells.
2. **Source sync progress** — the current `source sync: cloned=N ...`
   key-value output appears one line per skill with no progress
   indication. Users cannot gauge completion progress.
3. **Install results** — the current flat `skill → path` list repeats
   the skill name for every target. A tree-style grouped display is
   more scannable and informative.

## 2. Discovery Preview: Card-Style List

### 2.1 Unified Function

`print_discovered_skills` and `print_discovery_summary` MUST be
merged into a single function. Both `--list` mode and interactive
mode MUST use the same output format.

### 2.2 Format

The discovery preview MUST use a **card-style numbered list**, not
a `comfy-table` table. Each skill is rendered as:

- **Line 1:** number + skill name (bold).
- **Line 2+:** description (dimmed, indented), if non-empty.
  The description wraps at the terminal width minus the indent.
- Skills without descriptions show only the name line.

```text
  Found    4 skills in repository:

    1. vercel-composition-patterns
    2. vercel-react-best-practices
       React and Next.js performance optimization guidelines from Vercel
       Engineering. This skill should be used when writing, reviewing, or
       refactoring React/Next.js code to ensure optimal performance patterns.
    3. vercel-react-native-skills
    4. web-design-guidelines
       Review UI code for Web Interface Guidelines compliance. Use when
       asked to "review my UI", "audit design", or "check my site against
       best practices".
```

### 2.3 Number Formatting

The number field MUST be right-aligned with consistent width based
on the total count:

- 1–9 skills: `"    1. "` (4 spaces + number + dot + space)
- 10–99 skills: `"   10. "` (3 spaces + number + dot + space)

### 2.4 Description Indent

Description lines MUST be indented to align with the first character
of the skill name on line 1. For the default case (1–9 skills), this
is 7 spaces of leading indent.

### 2.5 Truncation

When the number of discovered skills exceeds **8**, the list MUST be
truncated and a footer appended:

```text
    ...
    8. some-skill
       Some description.

  ... and 12 more (use --list to see all)
```

When `--list` is active, ALL skills MUST be displayed without
truncation.

### 2.6 Coloring

| Element | Style |
| :--- | :--- |
| Skill name | bold (default foreground) |
| Description text | dimmed |
| Number | plain |
| `Found` | action prefix (bold cyan, right-aligned) |
| Truncation footer | dimmed |

### 2.7 Non-TTY / No-Color

When colors are disabled, skill names are plain text and descriptions
are undecorated. The numbered list structure is preserved.

### 2.8 JSON Mode

In `--json` mode, the existing JSON output contracts from Phase 2.5
(`--list` returns a JSON array of discovered skills) are unchanged.
The card-style list is a human-mode concern only.

## 3. Source Sync Progress: Step-Style

### 3.1 Replacement

The current per-skill `print_source_sync_summary()` call MUST be
replaced with a step-style progress indicator using `indicatif`.

### 3.2 Progress Bar Template

```text
  Syncing  [1/4] vercel-composition-patterns…
```

The progress bar MUST use:

- `ProgressBar::new(total_skills)` for a definite-length bar.
- Template: `"  {prefix} [{pos}/{len}] {msg}"`.
- Prefix: `ui.action_prefix("Syncing")`.
- Message: current skill name + `…`.

### 3.3 Step Lifecycle

For each skill in the install loop:

1. Set `pb.set_message(format!("{skill_id}…"))`.
2. Execute source sync.
3. Increment position: `pb.set_position(i + 1)`.

After all steps complete:

1. `pb.finish_and_clear()`.
2. Print a permanent styled summary line:

```text
  Syncing  4 synced, 0 failed
```

### 3.4 Non-TTY / Spinner-Disabled

When `spinner_enabled()` is false (non-TTY, CI, JSON), the progress
bar MUST NOT be created. Instead, a single summary line MUST be
printed after all sync operations complete (same format as step 5).

### 3.5 Error Handling

If a sync step fails, the progress bar MUST still advance. The
failure is accumulated in a counter and reported in the summary line.
The existing `source_sync_failure_error()` logic continues to govern
whether the install flow aborts or continues.

### 3.6 Scope

Step-style progress applies to:

- `install` (URL mode, per-skill sync loop)
- `install` (registry mode, single-skill sync)
- `apply` / `repair` (source sync phase — reuse the same progress
  rendering, but with the reactor-based sync function)
- `update --apply` (post-refresh sync phase)

For `apply`/`repair`, the sync is reactor-driven (concurrent). The
progress bar SHOULD still show `[pos/len]` updated as each skill
completes, using `ProgressBar::inc(1)` from the reactor callback.
If integrating a callback into the reactor is too invasive, a spinner
with a final summary is an acceptable fallback for `apply`/`repair`.

## 4. Install Results: Tree-Style Display

### 4.1 Grouped by Skill

Install result lines MUST be grouped by skill ID. Each group
displays the skill name once, followed by its target paths rendered
as a tree.

### 4.2 Tree Characters

| Position | Character |
| :--- | :--- |
| Non-last child | `├─` |
| Last child | `└─` |

### 4.3 Format

```text
  Install  ✓ vercel-composition-patterns
             ├─ ~/.claude/skills/vercel-composition-patterns (symlink)
             ├─ ~/.codex/skills/vercel-composition-patterns (symlink)
             └─ ~/.config/opencode/skills/vercel-composition-patterns (symlink)
           ✓ vercel-react-best-practices
             ├─ ~/.claude/skills/vercel-react-best-practices (symlink)
             ├─ ~/.codex/skills/vercel-react-best-practices (symlink)
             └─ ~/.config/opencode/skills/vercel-react-best-practices (symlink)
```

### 4.4 Tree Coloring

| Element | Style |
| :--- | :--- |
| `✓` | green (via `StatusSymbol::Success`) |
| Skill name | bold |
| Tree connectors (`├─`, `└─`) | dimmed |
| Target path | cyan |
| Mode label `(symlink)` / `(copy)` | dimmed |

### 4.5 Grouping Implementation

Target lines MUST be grouped by `skill_id` preserving insertion
order. The grouping logic uses a `Vec<(String, Vec<InstallTargetLine>)>`
(no new dependency on `IndexMap` required):

```rust
let mut groups: Vec<(String, Vec<&InstallTargetLine>)> = Vec::new();
for target in targets {
    if let Some(group) = groups.last_mut()
        .filter(|(id, _)| id == &target.skill_id)
    {
        group.1.push(target);
    } else {
        groups.push((target.skill_id.clone(), vec![target]));
    }
}
```

### 4.6 Summary Line

After the tree output, a blank line followed by the summary:

```text

  ✓ 4 skills installed to 3 agents, 0 conflicts
```

### 4.7 Apply / Repair Integration

The tree-style display MUST also be used by `apply` and `repair`
for their per-skill install output (replacing the current flat
`print_install_applied_line` calls in `reconcile.rs`).

### 4.8 Dry-Run Multi-Skill Preview

When `install --dry-run` resolves multiple selected skills in URL mode,
human output MUST render two titled tables:

1. `Skill / Version / Source`
2. `Install Targets`

Formatting requirements:

- The title line for each table MUST be rendered as a standalone heading
  before the table.
- Each table block MUST be left-indented by 4 spaces.
- The `Skill / Version / Source` table MUST default to the first 8
  selected skills.
- The `Install Targets` table MUST include only `Agent`, `Path`, and
  `Mode` columns (no per-skill column).
- When `--list` is combined with `--dry-run`, ALL selected skills MUST
  be shown (no truncation).
- JSON mode behavior remains machine-readable and MUST NOT render tables.

## 5. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **IUX-001** | Builder | **P0** | Discovery preview MUST use card-style numbered list per Section 2.2. | `--list` and interactive mode both show numbered card format. |
| **IUX-002** | Builder | **P0** | `print_discovered_skills` and `print_discovery_summary` MUST be merged into one function. | Only one discovery display function exists. |
| **IUX-003** | Builder | **P0** | Descriptions MUST be dimmed and indented below the skill name. | Description on a separate indented line, styled dimmed. |
| **IUX-004** | Builder | **P0** | Source sync MUST use step-style progress `[pos/len]` in TTY mode. | Progress bar shows `[1/4] skill-name…` pattern. |
| **IUX-005** | Builder | **P0** | Source sync MUST print a styled summary line after completion. | `Syncing  N synced, M failed` line present. |
| **IUX-006** | Builder | **P0** | Install results MUST use tree-style grouped display per Section 4.3. | Tree connectors (`├─`, `└─`) present; skill name appears once per group. |
| **IUX-007** | Builder | **P0** | Tree connectors and mode labels MUST be dimmed; paths MUST be cyan. | Color verification in styled output. |
| **IUX-008** | Builder | **P1** | `apply`/`repair` MUST use the same tree-style display for install lines. | `reconcile.rs` uses tree format. |
| **IUX-009** | Builder | **P1** | `install --dry-run` multi-skill preview MUST render titled indented skill/target tables; skill table defaults to 8 rows and `--list` shows all. Target table MUST be `Agent/Path/Mode` only. | Dry-run output shows two titled tables with 4-space indentation; truncation/default + `--list` full behavior validated; target table excludes skill column. |

## 6. Backward Compatibility

| Existing Feature | Phase 2.9 Behavior |
| :--- | :--- |
| `install --list --json` | JSON output unchanged. |
| `install --json` | JSON output unchanged. |
| `install --dry-run` | Multi-skill dry-run uses titled table preview (`Skill / Version / Source` + `Install Targets`) with default 8-row skill truncation; `--dry-run --list` shows all selected skills. |
| `EDEN_SKILLS_TEST_CONFIRM` | Test env var still works for non-interactive flows. |
| Spinner for cloning | Cloning spinner preserved. Discovery preview follows it. |
