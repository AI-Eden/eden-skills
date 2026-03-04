# SPEC_TABLE_RENDERING.md

Table rendering infrastructure and policies for `eden-skills`.

**Related contracts:**

- `spec/phase2.5/SPEC_CLI_UX.md` (visual design language)
- `spec/phase2.7/SPEC_OUTPUT_POLISH.md` (color library and `--color` flag)

## 1. Purpose

Several commands produce multi-record, multi-attribute output that is
currently rendered as raw `key=value` lines. This spec introduces
terminal-aware table rendering so that structured data is presented in
a scannable, aligned format that adapts to terminal width.

## 2. Technology Stack

### 2.1 Dependency

| Action | Crate | Version | Purpose |
| :--- | :--- | :--- | :--- |
| **Add** | `comfy-table` | `7` | Terminal-aware table rendering with column width control |

### 2.2 Rationale

- Lightweight and mature; used by CLI tools across the Rust ecosystem.
- Does not impose its own color system — accepts pre-colored strings
  from `owo-colors`, preserving the existing color infrastructure.
- Built-in terminal width detection (`crossterm`) with automatic column
  wrapping.
- Supports per-column alignment, min/max width constraints, and
  content wrapping.

### 2.3 No Additional Dependencies

`comfy-table` is the only new dependency for table rendering.
`owo-colors`, `indicatif`, and `dialoguer` remain unchanged.

## 3. Table Construction API

### 3.1 `UiContext` Extension

The `UiContext` struct in `ui.rs` MUST be extended with a table factory
method:

```rust
impl UiContext {
    /// Creates a new table pre-configured with the current color and
    /// terminal-width policies.
    pub fn table(&self, headers: &[&str]) -> comfy_table::Table;
}
```

The returned `Table` MUST be configured as follows:

| Property | TTY (human mode) | Non-TTY (piped / CI) |
| :--- | :--- | :--- |
| Border style | `UTF8_FULL_CONDENSED` | `ASCII_FULL_CONDENSED` |
| Header styling | Bold (via `owo-colors`) | Plain text |
| Terminal width | Auto-detected | `80` (fallback) |
| Content wrapping | Enabled | Enabled |

When `--json` is set, commands MUST NOT call `UiContext::table()` at all.
Table output is a human-mode concern only.

### 3.2 Color Integration

Table header cells SHOULD be styled bold when colors are enabled.
Table cell content MAY contain pre-colored strings (e.g., status symbols
from `UiContext::status_symbol()`). `comfy-table` preserves ANSI
sequences in cell content by default; no additional configuration is
needed.

When colors are disabled (`--color never`, `NO_COLOR`, non-TTY), header
cells MUST be plain text. The `owo_colors::set_override(false)` global
flag (already set by `configure_color_output`) ensures that any
`OwoColorize` calls produce plain strings.

### 3.3 Column Width Strategy

Each table definition (Section 5) specifies per-column width constraints.
The general policy:

- **Fixed-width columns** (Action, Mode, Status): `set_constraint`
  with `Width::Fixed(N)`.
- **Elastic columns** (Skill ID, Agent, Description): no constraint;
  `comfy-table` distributes remaining width.
- **Long-content columns** (Path, Repo URL): content is pre-processed
  with semantic abbreviation (Section 4) before insertion. Column uses
  `Width::Percentage` or unconstrained with wrapping as fallback.

## 4. Long Content Strategy

### 4.1 Semantic Abbreviation

Before inserting long values into table cells, the CLI MUST apply
semantic abbreviation:

| Input | Abbreviated Output | Rule |
| :--- | :--- | :--- |
| `/home/eden/.claude/skills/browser-tool` | `~/.claude/skills/browser-tool` | Replace `$HOME` prefix with `~` |
| `https://github.com/owner/repo.git` | `owner/repo` | Strip scheme, host, `.git` suffix |
| `https://github.com/owner/repo` | `owner/repo` | Strip scheme, host |
| `/absolute/path/without/home` | `/absolute/path/without/home` | No change (not under `$HOME`) |
| `registry://official/skill-name` | `registry://official/skill-name` | No change (registry URIs are short) |

### 4.2 Utility Functions

Two utility functions MUST be added to the CLI crate (not core — these
are presentation-only concerns):

```rust
/// Replaces the $HOME prefix in a path with `~`.
pub fn abbreviate_home_path(path: &str) -> String;

/// Extracts `owner/repo` from a GitHub URL.
/// Returns the input unchanged if it is not a recognized GitHub URL.
pub fn abbreviate_repo_url(url: &str) -> String;
```

These functions MUST be pure (no I/O, no side effects) and covered by
unit tests.

### 4.3 Wrapping Fallback

After semantic abbreviation, if a cell value still exceeds the column's
allocated width, `comfy-table`'s built-in content wrapping MUST handle
the overflow. No manual truncation with `…` is required — `comfy-table`
wraps to the next line within the cell.

## 5. Table Definitions

### 5.1 `list` Command

Replaces the current `skill id=X mode=Y repo=Z ...` output.

**Header:** `Skill | Mode | Source | Agents`

| Column | Content | Width | Notes |
| :--- | :--- | :--- | :--- |
| Skill | `skill.id` | elastic | |
| Mode | `skill.install.mode` (`symlink` / `copy`) | fixed 7 | |
| Source | `abbreviate_repo_url(skill.source.repo)` | elastic | Abbreviated |
| Agents | Comma-joined `agent` labels from `skill.targets` | elastic | e.g. `claude-code, cursor` |

Skills with `safety.no_exec_metadata_only = true` MUST append
`(metadata-only)` to the Agents column.

**Example output:**

```text
  Skills   5 configured

 Skill              Mode     Source                    Agents
 browser-tool       symlink  vercel-labs/agent-skills  claude-code, cursor
 filesystem-tool    symlink  vercel-labs/agent-skills  claude-code, cursor
 github-tool        copy     vercel-labs/agent-skills  claude-code (metadata-only)
 search-tool        symlink  user/search-skills        cursor
 custom-devops      symlink  ./local-skills            custom:/opt/agent
```

### 5.2 `install --dry-run` Targets

Replaces the current `target agent=X environment=Y path=Z` output.

The dry-run output MUST display a metadata header followed by a targets
table.

**Metadata header** (styled key-value lines):

```text
  Dry Run  install preview

  Skill:   browser-tool
  Version: main
  Source:   vercel-labs/agent-skills (skills/browser-tool)
```

**Targets table header:** `Agent | Path | Mode`

| Column | Content | Width |
| :--- | :--- | :--- |
| Agent | `agent_kind_label` | elastic |
| Path | `abbreviate_home_path(resolved_path)` | elastic |
| Mode | install mode | fixed 7 |

### 5.3 `install --list` Discovered Skills

Replaces the current plain bullet list.

**Header:** `# | Name | Description`

| Column | Content | Width |
| :--- | :--- | :--- |
| # | 1-indexed number | fixed 3 |
| Name | `skill.name` | elastic |
| Description | `skill.description` (may be empty) | elastic |

When the number of discovered skills exceeds 8, the table MUST be
truncated and a footer line MUST be appended:

```text
  ... and N more (use --list to see all)
```

### 5.4 `plan` (Threshold: > 5 Actions)

When a plan contains **more than 5 actions**, the output SHOULD switch
from the text format (Section 5.4a) to a table format.

When a plan contains **5 or fewer actions**, the text format defined in
`SPEC_OUTPUT_UPGRADE.md` Section 4.3 MUST be used.

**Table header:** `Action | Skill | Target | Mode`

| Column | Content | Width |
| :--- | :--- | :--- |
| Action | `action_label`, colored by type | fixed 8 |
| Skill | `skill_id` | elastic |
| Target | `abbreviate_home_path(target_path)` | elastic |
| Mode | install mode | fixed 7 |

Conflict reasons MUST be rendered as indented lines below the table:

```text
  Conflicts:
    github-tool → ~/.claude/skills/github-tool
      reason: target exists but is not a symlink
```

### 5.5 `update` Registry Results

Replaces the current single-line `registry sync: name=status ...` output.

**Header:** `Registry | Status | Detail`

| Column | Content | Width |
| :--- | :--- | :--- |
| Registry | `result.name` | elastic |
| Status | colored status (`cloned` green, `updated` green, `skipped` dim, `failed` red) | fixed 8 |
| Detail | `result.detail` or empty | elastic |

### 5.6 `doctor` Summary Table (Conditional)

When doctor findings exceed **3 items**, a compact summary table MUST
be displayed before the detailed findings cards (defined in
`SPEC_OUTPUT_UPGRADE.md` Section 4.2).

**Header:** `Sev | Code | Skill`

| Column | Content | Width |
| :--- | :--- | :--- |
| Sev | `✗` (red) or `!` (yellow) | fixed 3 |
| Code | finding code (e.g., `SOURCE_MISSING`) | elastic |
| Skill | `skill_id` | elastic |

When findings are 3 or fewer, the summary table is omitted and only
the detail cards are shown.

## 6. Non-TTY Degradation

When stdout is not a TTY (piped, redirected, CI):

- Tables MUST use ASCII borders (`ASCII_FULL_CONDENSED`) instead of
  Unicode box-drawing characters.
- No ANSI color codes in any cell content.
- Terminal width MUST fall back to `80` columns.
- Tables MUST still be rendered (not suppressed) — they provide
  structure even in plain text.

## 7. JSON Mode Exclusion

When `--json` is set:

- Tables MUST NOT be rendered. The JSON output contracts from
  Phase 1/2/2.5/2.7 remain the sole output.
- No `comfy-table` code paths are reached in JSON mode.

## 8. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **TBL-001** | Builder | **P0** | `comfy-table` MUST be added as a dependency to `eden-skills-cli`. | `Cargo.toml` lists `comfy-table`. Build succeeds. |
| **TBL-002** | Builder | **P0** | `UiContext` MUST provide a `table()` factory method that respects color and terminal-width policies. | `UiContext::table()` returns configured `Table`. Non-TTY uses ASCII borders. |
| **TBL-003** | Builder | **P0** | `list` MUST render skills as a table with columns Skill, Mode, Source, Agents. | `eden-skills list` output is a formatted table. |
| **TBL-004** | Builder | **P0** | `install --dry-run` MUST render targets as a table with columns Agent, Path, Mode. | Dry-run output includes a targets table. |
| **TBL-005** | Builder | **P1** | `install --list` MUST render discovered skills as a numbered table. | `--list` output shows `#`, `Name`, `Description` columns. |
| **TBL-006** | Builder | **P1** | `plan` with > 5 actions MUST render as a table. ≤ 5 actions use text format. | Plan with 6+ items shows table; plan with 3 items shows text. |
| **TBL-007** | Builder | **P1** | `update` MUST render registry sync results as a table. | `update` output shows Registry, Status, Detail columns. |

## 9. Backward Compatibility

| Existing Feature | Phase 2.8 Behavior |
| :--- | :--- |
| `--json` output | Unchanged. Tables never appear in JSON mode. |
| Non-TTY piped output | Tables rendered with ASCII borders; no color. |
| `NO_COLOR` / `FORCE_COLOR` / `CI` env vars | Unchanged. Table styling follows existing color policy. |
| `--color` flag | Unchanged. Table header/cell styling follows resolved color mode. |
