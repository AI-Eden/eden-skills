# SPEC_OUTPUT_UPGRADE.md

Full-command output upgrade, UiContext unification, and error format
refinement for `eden-skills`.

**Related contracts:**

- `spec/phase2.5/SPEC_CLI_UX.md` (visual design language — Sections 4 and 5)
- `spec/phase2.7/SPEC_OUTPUT_POLISH.md` (owo-colors, `--color` flag, error format)
- `spec/phase2.8/SPEC_TABLE_RENDERING.md` (table infrastructure)

## 1. Purpose

Phase 2.5 defined a visual design language (symbols, action prefixes,
spinners, command-specific output). Phase 2.7 migrated the color
backend to `owo-colors` and refined error messages. However, the
majority of commands still emit raw `key=value` output that does not
match the spec-defined ideal. This spec closes that gap:

1. **Category A** — commands whose spec-defined output already exists
   but was never implemented.
2. **Category B** — commands that need new output designs (primarily
   table-based), defined here for the first time.
3. **UiContext unification** — all commands MUST use `UiContext` for
   human-mode output.
4. **Error format alignment** — the hint format is updated to match
   the spec-defined `→` style.

## 2. UiContext Unification Rule

### 2.1 Mandate

Every human-mode output path in every command MUST obtain a `UiContext`
instance and use it for symbols, action prefixes, and table construction.

Commands that currently bypass `UiContext`:

- `plan`
- `apply` / `repair`
- `doctor`
- `init`
- `list`
- `update`

After Phase 2.8, the only command functions that directly call
`println!()` without `UiContext` mediation are JSON-mode paths.

### 2.2 UiContext Lifetime

For commands that do not currently create a `UiContext`, the context
MUST be created at the top of the command function:

```rust
let ui = UiContext::from_env(options.json);
```

The `UiContext` is passed by reference to all output helper functions.

## 3. Action Color Palette

The following action-to-color mapping MUST be used consistently across
all commands:

| Action | Color | Usage |
| :--- | :--- | :--- |
| `create` | green | New target installation |
| `update` | cyan | Existing target refresh |
| `remove` | red | Target or skill removal |
| `conflict` | yellow | State disagreement |
| `noop` | dim | No change needed |

Action labels used as `action_prefix` (e.g., `Syncing`, `Install`,
`Remove`, `Doctor`, `Plan`) follow the existing Phase 2.5 convention:
right-aligned, bold cyan.

Status count values in summary lines SHOULD be colored when non-zero:
green for positive outcomes (created, cloned), red for failures, yellow
for conflicts, dim for noops.

## 4. Command Output Specifications

### 4.1 `apply` / `repair`

> **Origin:** `spec/phase2.5/SPEC_CLI_UX.md` Section 5.2 defines the
> ideal `apply` / `repair` output. The current implementation does not
> conform to it.

#### Source Sync Summary

**Before (current):**

```txt
source sync: cloned=1 updated=2 skipped=0 failed=0
```

**After:**

```txt
  Syncing  1 cloned, 2 updated, 0 skipped, 0 failed
```

`Syncing` MUST be a styled action prefix. Count values MUST be
colored: cloned/updated green when > 0, failed red when > 0,
skipped dim.

#### Safety Summary

**Before (current):**

```txt
safety summary: permissive=3 non_permissive=0 unknown=0 risk_labeled=0 no_exec=1
```

**After:**

```txt
  Safety   3 permissive, 0 risk flags, 1 no-exec
```

`Safety` MUST be a styled action prefix. `risk_labeled` and
`non_permissive` are merged into a single `risk flags` count
(displayed in yellow when > 0). `no_exec` is displayed in dim.

#### Per-Skill Install Lines

For each plan item with action `Create` or `Update` that is executed,
the CLI MUST emit a status line:

```txt
  Install  ✓ browser-tool → ~/.claude/skills/browser-tool (symlink)
           ✓ browser-tool → ~/.cursor/skills/browser-tool (symlink)
```

For skipped no-exec items:

```txt
           · github-tool (skipped: metadata-only)
```

For items with action `Remove` (from lock diff):

```txt
  Remove   ✓ old-skill
```

The first `Install` or `Remove` line uses the action prefix. Subsequent
lines for the same action use indentation to align with the first.

#### Apply/Repair Summary

**Before (current):**

```txt
apply summary: create=1 update=0 noop=2 conflict=0 skipped_no_exec=0 removed=1
```

**After:**

```txt
  ✓ 1 created, 0 updated, 2 noop, 0 conflicts, 1 removed
```

The `✓` MUST be green. Individual count values MUST be colored per
Section 3.

#### Verification

**Before (current):**

```txt
apply verification: ok
```

**After:**

```txt
  ✓ Verification passed
```

### 4.2 `doctor`

> **Origin:** `spec/phase2.5/SPEC_CLI_UX.md` Section 5.3 defines the
> ideal `doctor` output with severity-colored symbols, indented messages,
> and `→` remediation. The current implementation emits a single-line
> key=value dump per finding.

#### Header

**Before (current):**

```txt
doctor: detected 2 issue(s)
```

**After:**

```txt
  Doctor   2 issues detected
```

`Doctor` MUST be a styled action prefix.

When no issues are detected:

```txt
  Doctor   ✓ no issues detected
```

#### Findings Cards

**Before (current):**

```txt
  code=SOURCE_MISSING severity=error skill=browser-tool target=/path message=... remediation=...
```

**After:**

```txt
  ✗ [SOURCE_MISSING] browser-tool
    Source path does not exist: ~/.eden-skills/skills/browser-tool
    → Run `eden-skills apply` to sync sources.
```

Each finding MUST be rendered as a card:

1. **Line 1:** severity symbol (`✗` red for error, `!` yellow for
   warning) + `[CODE]` + `skill_id`.
2. **Line 2:** message text, indented 4 spaces.
3. **Line 3:** `→` (dimmed) + remediation text, indented 4 spaces.
4. **Blank line** between cards.

#### Summary Table (Conditional)

When findings exceed 3, a summary table is prepended per
`SPEC_TABLE_RENDERING.md` Section 5.6.

### 4.3 `plan`

> **Origin:** `spec/phase2.5/SPEC_CLI_UX.md` Section 5.4 defines the
> ideal `plan` output with action labels, colored types, and `→` target
> paths. The current implementation has no header and uses unaligned
> plain text with `->`.

#### Header <!-- markdownlint-disable-line MD009 -->

**Before (current):**

```txt
(no header)
```

**After:**

```txt
  Plan     4 actions
```

`Plan` MUST be a styled action prefix. When the plan is empty:

```txt
  Plan     ✓ 0 actions (up to date)
```

#### Text Format (≤ 5 Actions)

**Before (current):**

```txt
create browser-tool /home/eden/.eden-skills/skills/browser-tool -> /home/eden/.claude/skills/browser-tool (symlink)
```

**After:**

```txt
  create   browser-tool → ~/.claude/skills/browser-tool (symlink)
  noop     filesystem-tool → ~/.cursor/skills/filesystem-tool
  conflict github-tool → ~/.claude/skills/github-tool
           reason: target exists but is not a symlink
```

Changes:

- Action label right-aligned to 8 characters and colored per Section 3.
- `→` replaces `->`.
- Source path is omitted (redundant — the skill ID implies the source).
- Target path abbreviated with `~`.
- Conflict reasons indented below the action line.

#### Table Format (> 5 Actions)

Per `SPEC_TABLE_RENDERING.md` Section 5.4.

### 4.4 `init`

> **Origin:** `spec/phase2.5/SPEC_CLI_UX.md` Section 5.6 defines the
> ideal `init` output with `✓` symbol and a "Next steps" guidance block.
> The current implementation emits `init: wrote <path>` with no guidance.

**Before (current):**

```txt
init: wrote /home/eden/.eden-skills/skills.toml
```

**After:**

```txt
  ✓ Created config at ~/.eden-skills/skills.toml

  Next steps:
    eden-skills install <owner/repo>     Install skills from GitHub
    eden-skills list                     Show configured skills
    eden-skills doctor                   Check installation health
```

The `✓` MUST be green. The "Next steps" block MUST be styled with
dimmed command descriptions. Path MUST be abbreviated with `~`.

### 4.5 `install` — Per-Skill Install Results

> **Origin:** `spec/phase2.5/SPEC_CLI_UX.md` Section 5.1 defines the
> ideal install output with per-skill per-target `✓ skill → path` lines
> and a summary including agent count. The current implementation only
> emits a final count line.

#### URL Mode (Remote and Local)

**Before (current):**

```txt
source sync: cloned=0 updated=1 skipped=0 failed=0
✓ install: 3 skill(s) status=installed
```

**After:**

```txt
  Install  ✓ browser-tool → ~/.claude/skills/browser-tool (symlink)
           ✓ browser-tool → ~/.cursor/skills/browser-tool (symlink)
           ✓ filesystem-tool → ~/.claude/skills/filesystem-tool (symlink)
           ✓ filesystem-tool → ~/.cursor/skills/filesystem-tool (symlink)

  ✓ 2 skills installed to 2 agents, 0 conflicts
```

The `Install` action prefix MUST be used for the first line. Subsequent
lines are indented to align. The final summary MUST include skill count,
agent count, and conflict count.

Agent count is the number of distinct agent targets across all installed
skills.

#### Registry Mode

Same format as URL mode. The source sync summary line is also upgraded
per Section 4.1.

### 4.6 `install` — Discovery Summary

> **Origin:** `spec/phase2.5/SPEC_CLI_UX.md` Section 5.1 defines the
> discovery summary with `Found` action prefix and numbered entries.
> The current implementation shows unnumbered entries without an action
> prefix.

**Before (current):**

```txt
Found 3 skills in repository:
  browser-tool — Browser automation
  filesystem-tool — File system operations
```

**After:**

```txt
  Found    3 skills in repository:

    1. browser-tool        — Browser automation
    2. filesystem-tool     — File system operations
    3. github-tool         — GitHub API integration
```

`Found` MUST be a styled action prefix. Items MUST be numbered. Skill
names MUST be left-aligned with consistent padding. The description
follows `—` (em dash).

When the number of skills exceeds 8, truncation applies per
`SPEC_TABLE_RENDERING.md` Section 5.3.

### 4.7 `list`

> **Origin:** `spec/phase2.5/SPEC_CLI_UX.md` Section 5.5 defines a
> manually-aligned text format. The current implementation does not
> conform to it. Phase 2.8 supersedes the Section 5.5 design with a
> `comfy-table` table for better alignment and terminal-width
> adaptability.

**Before (current):**

```txt
list: 5 skill(s)
skill id=browser-tool mode=symlink repo=https://github.com/... ref=main subpath=skills/browser-tool
  verify enabled=true checks=path-exists,is-symlink,target-resolves
  target agent=claude-code path=/home/eden/.claude/skills/browser-tool
```

**After:**

Table format per `SPEC_TABLE_RENDERING.md` Section 5.1.

The `Skills  N configured` header line MUST precede the table.

When the config is empty:

```txt
  Skills   0 configured
```

### 4.8 `update`

**Before (current):**

```txt
registry sync: official=updated forge=skipped (0 failed) [1.2s]
```

**After:**

Table format per `SPEC_TABLE_RENDERING.md` Section 5.5.

The table MUST be preceded by an action-prefix header and followed by
a timing line:

```txt
  Update   2 registries synced

 Registry   Status    Detail
 official   updated
 forge      skipped

  ✓ 0 failed [1.2s]
```

### 4.9 `install --dry-run`

**Before (current):**

```txt
install dry-run: skill=X version=V repo=R ref=F subpath=S
  target agent=claude-code environment=local path=/home/eden/.claude/skills/X
```

**After:**

Metadata header + targets table per `SPEC_TABLE_RENDERING.md`
Section 5.2.

### 4.10 `remove` — Summary (Existing)

The `remove` command already uses `UiContext` and emits styled output.
No changes are required. This section documents the current output for
completeness:

```txt
  Remove   ✓ browser-tool
           ✓ code-review

  ✓ 2 skills removed
```

## 5. Error Format Alignment

### 5.1 Hint Prefix Change

> **Origin:** `spec/phase2.7/SPEC_OUTPUT_POLISH.md` Section 5.1 defines
> the error display format with `→` hint prefix. The current
> implementation uses `hint:` in purple instead of `→` in dimmed, and
> does not abbreviate paths with `~`.

**Before (current implementation):**

```txt
error: config file not found: /home/eden/.eden-skills/skills.toml

 hint: Run 'eden-skills init' to create a new config.
```

**After:**

```txt
error: config file not found: ~/.eden-skills/skills.toml

  → Run 'eden-skills init' to create a new config.
```

Changes:

1. `hint:` (purple) is replaced with `→` (dimmed).
2. Indent changes from ` hint: ` (1 space + word) to ` → ` (2 spaces
   - arrow + space).
3. Paths in error messages MUST be abbreviated with `~`.

### 5.2 Implementation

The `print_error` function in `main.rs` MUST be updated:

```rust
if let Some(hint) = hint {
    if colors_enabled {
        eprintln!("  {} {hint}", "→".dimmed());
    } else {
        eprintln!("  → {hint}");
    }
}
```

The `split_hint` function and the `\nhint:` delimiter used in
`EdenError` string construction remain unchanged internally. Only the
display format changes.

### 5.3 Path Abbreviation in Errors

Error messages that include file paths SHOULD use
`abbreviate_home_path()` for user-facing display. This applies to:

- Config file not found errors.
- Storage directory errors.
- Target path permission errors.

Error messages in `--json` mode MUST continue to use absolute paths
for machine parseability.

## 6. Warning Format

Warnings emitted via `eprintln!("warning: ...")` MUST be updated to
use `UiContext`-aware formatting:

**Before:**

```txt
warning: no registries configured; skipping update
```

**After:**

```txt
  warning: no registries configured; skipping update
```

When colors are enabled, `warning:` MUST be styled in yellow bold.
The message MUST be indented with 2 spaces for visual consistency
with action prefixes.

## 7. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **OUP-001** | Builder | **P0** | `apply` human-mode output MUST match Section 4.1. | Source sync, safety, per-skill, summary, verification lines match spec format. |
| **OUP-002** | Builder | **P0** | `repair` human-mode output MUST match Section 4.1 (same format as `apply`). | Repair output uses same styled format. |
| **OUP-003** | Builder | **P0** | `doctor` human-mode output MUST match Section 4.2 (header + finding cards). | Doctor findings show severity symbols, indented message, `→` remediation. |
| **OUP-004** | Builder | **P0** | `plan` human-mode output MUST match Section 4.3 (header + colored action labels). | Plan output shows `Plan N actions` header and colored, aligned action lines. |
| **OUP-005** | Builder | **P0** | `init` output MUST include `✓` symbol, path abbreviation, and Next steps block. | `init` shows success symbol and 3-line guidance block. |
| **OUP-006** | Builder | **P0** | `install` URL-mode MUST emit per-skill per-target install lines with `✓ skill → path`. | Install output shows each target with status symbol. |
| **OUP-007** | Builder | **P0** | `install` discovery summary MUST use `Found` action prefix with numbered list. | Discovery output is numbered and action-prefixed. |
| **OUP-008** | Builder | **P0** | `list` MUST render as a table per `SPEC_TABLE_RENDERING.md` Section 5.1. | `list` output is a formatted table with Skill/Mode/Source/Agents columns. |
| **OUP-009** | Builder | **P0** | `install --dry-run` targets MUST render as a table per `SPEC_TABLE_RENDERING.md` Section 5.2. | Dry-run shows metadata header and targets table. |
| **OUP-010** | Builder | **P1** | `install --list` MUST render as a numbered table per `SPEC_TABLE_RENDERING.md` Section 5.3. | `--list` shows numbered table with Name and Description columns. |
| **OUP-011** | Builder | **P1** | `plan` with > 5 actions MUST render as a table per `SPEC_TABLE_RENDERING.md` Section 5.4. | Plan with 6 actions shows table format; plan with 3 shows text format. |
| **OUP-012** | Builder | **P1** | `update` MUST render registry results as a table per `SPEC_TABLE_RENDERING.md` Section 5.5. | Update output shows registry table. |
| **OUP-013** | Builder | **P0** | Error hint prefix MUST use `→` (dimmed) instead of `hint:` (purple). | Error output shows `→` not `hint:`. |
| **OUP-014** | Builder | **P0** | Error paths MUST be abbreviated with `~` in human mode. | Missing config error shows `~/.eden-skills/...` not absolute path. |
| **OUP-015** | Builder | **P0** | All commands MUST create `UiContext` for human-mode output (Section 2). | No command uses `println!` without `UiContext` except JSON paths. |
| **OUP-016** | Builder | **P0** | Action colors MUST follow Section 3 palette. | `create` is green, `remove` is red, `conflict` is yellow, `noop` is dim. |
| **OUP-017** | Builder | **P1** | Warnings MUST use `warning:` in yellow bold with 2-space indent. | Warning lines start with `warning:` styled. |
| **OUP-018** | Builder | **P0** | `install` final summary MUST include skill count, agent count, conflict count. | Summary line shows `N skills installed to M agents, K conflicts`. |
| **OUP-019** | Builder | **P0** | `apply`/`repair` source sync, safety, and summary MUST use action prefixes. | `Syncing`, `Safety` action prefixes are present and styled. |
| **OUP-020** | Builder | **P1** | `doctor` summary table MUST be shown when findings > 3. | Doctor with 4+ findings shows summary table before detail cards. |

## 8. Backward Compatibility

| Existing Feature | Phase 2.8 Behavior |
| :--- | :--- |
| `--json` output | Unchanged. All JSON schemas preserved. |
| Exit codes | Unchanged (1/2/3 semantics preserved). |
| `--strict` behavior | Unchanged. Strict mode still triggers on conflicts. |
| `remove` output | Already styled in Phase 2.7. No change. |
| `install` spinner | Already implemented. No change. |
| Error exit behavior | Error types and matching unchanged; only display format updated. |
