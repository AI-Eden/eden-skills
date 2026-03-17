# SPEC_LIST_SOURCE.md

Replace the `list` table's `Path` column with a human-friendly `Source` column.

**Related contracts:**

- `spec/phase2.97/SPEC_TABLE_STYLE.md` Section 6.1 (superseded by this spec)
- `spec/phase2.5/SPEC_INSTALL_URL.md` (source format conventions)

## 1. Problem Statement

The `list` command currently renders a `Path` column showing the
resolved repo-cache path (e.g.,
`~/.eden-skills/skills/.repos/github.com_vercel-labs_agent-skills@main/skills/react-best-practices`).
This path is an internal implementation detail of the storage layer
and provides no actionable information to the user.

The `install --dry-run` command already renders a concise `Source`
column in the format `owner/repo (subpath)` using `abbreviate_repo_url`
and `abbreviate_home_path`. The `list` command MUST adopt the same
format for consistency.

## 2. Source Column Format

### 2.1 Format Definition

The `Source` column value for each skill MUST be computed as:

```rust
let repo_display = abbreviate_home_path(&abbreviate_repo_url(&skill.source.repo));
let source = format!("{repo_display} ({})", skill.source.subpath);
```

Both `abbreviate_repo_url` and `abbreviate_home_path` are already
exported from `crates/eden-skills-cli/src/ui/format.rs`.

### 2.2 Examples

| `skill.source.repo` | `skill.source.subpath` | Rendered Source |
| :--- | :--- | :--- |
| `https://github.com/vercel-labs/agent-skills.git` | `skills/react-best-practices` | `vercel-labs/agent-skills (skills/react-best-practices)` |
| `https://github.com/anthropics/courses.git` | `.` | `anthropics/courses (.)` |
| `/home/eden/local-skills` | `my-skill` | `~/local-skills (my-skill)` |

### 2.3 Styling

When `UiContext::colors_enabled()` is true, the `Source` cell MUST
use **cyan** foreground via `ui.styled_cyan()`, consistent with the
`install --dry-run` Source column styling.

## 3. Table Header Change

The `list` command table headers MUST change from:

```text
Skill | Mode | Path | Agents
```

to:

```text
Skill | Mode | Source | Agents
```

### 3.1 Example Output

```text
  Skills  2 configured

 ┌────────────────────────────────┬─────────┬──────────────────────────────────────────────────────┬──────────────┐
 │ Skill                          ┆ Mode    ┆ Source                                               ┆ Agents       │
 ╞════════════════════════════════╪═════════╪══════════════════════════════════════════════════════╪══════════════╡
 │ vercel-react-best-practices    ┆ symlink ┆ vercel-labs/agent-skills (skills/react-best-practices)┆ cursor       │
 │ web-design-guidelines          ┆ symlink ┆ vercel-labs/agent-skills (skills/web-design-guidelines)┆ claude-code  │
 └────────────────────────────────┴─────────┴──────────────────────────────────────────────────────┴──────────────┘
```

## 4. JSON Output

The `list --json` output already includes a `source` object with
`repo`, `ref`, and `subpath` fields. No change is required to the
JSON schema.

## 5. Implementation Notes

### 5.1 Removed Dependency

The `list` function currently calls `resolve_skill_source_path` to
compute the `Path` column. After this change, that call is no longer
needed for the table rendering path. However, `storage_root` is still
used by the JSON code path, so it SHOULD be retained.

### 5.2 Import Changes

`config_ops.rs` MUST add `use crate::ui::abbreviate_repo_url;` (or
use the re-export from `crate::ui`). The existing
`abbreviate_home_path` import is already present.

## 6. Backward Compatibility

| Existing Feature | Phase 2.98 Behavior |
| :--- | :--- |
| `list` table column count | Unchanged (4 columns) |
| `list` table column position | Unchanged (3rd column) |
| `list` table column header | Changed: `Path` → `Source` |
| `list` table column content | Changed: repo-cache path → `owner/repo (subpath)` |
| `list --json` | Unchanged |
| `--color never` / non-TTY | Source column renders as plain text |

## 7. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **LSR-001** | Builder | **P0** | `list` table MUST show `Source` header instead of `Path`. | `list` output contains `Source` header. |
| **LSR-002** | Builder | **P0** | `Source` column MUST render `abbreviate_repo_url(repo) (subpath)` format. | Source column matches `install --dry-run` format. |
| **LSR-003** | Builder | **P1** | `Source` column MUST use cyan styling when colors are enabled. | Source cells contain ANSI cyan sequences in TTY mode. |
