# SPEC_DOCTOR_UX.md

Doctor output enhancements: `--no-warning` flag, severity column
rename, and severity coloring.

**Related contracts:**

- `spec/phase2.97/SPEC_TABLE_STYLE.md` (table styling infrastructure)
- `spec/phase2.97/SPEC_CACHE_CLEAN.md` Section 4 (doctor findings)
- `spec/phase2.97/SPEC_DOCKER_MANAGED.md` Section 5 (doctor findings)

## 1. Problem Statement

Three issues with the current `doctor` output:

1. **No warning filter.** Users who want to focus on errors must
   visually skip warning-level findings. There is no way to suppress
   them programmatically.
2. **Inconsistent naming.** The summary table uses `Sev` as the column
   header and `warn` as a cell value, while the internal data model
   stores `"warning"`. The abbreviation is inconsistent with `error`
   and `info` which are not abbreviated.
3. **No severity coloring.** The `Level` column in the summary table
   is rendered as plain text. Other tables (e.g., `update` status)
   already colorize semantic values.

## 2. `--no-warning` Flag

### 2.1 CLI Argument

A new `DoctorArgs` struct MUST replace the current `CommonArgs` for
the `doctor` subcommand:

```rust
#[derive(Debug, Clone, Args)]
struct DoctorArgs {
    #[arg(long, default_value = DEFAULT_CONFIG_PATH, hide_default_value = true,
          help = "Path to skills.toml config file [default: ~/.eden-skills/skills.toml]")]
    config: String,
    #[arg(long, help = "Exit with error on drift or warnings")]
    strict: bool,
    #[arg(long, help = "Output machine-readable JSON")]
    json: bool,
    #[arg(long, help = "Hide warning-level findings from output")]
    no_warning: bool,
}
```

### 2.2 Dispatch

`Commands::Doctor(DoctorArgs)` MUST pass the `no_warning` field to
`commands::doctor()`. The function signature becomes:

```rust
pub fn doctor(
    config_path: &str,
    options: CommandOptions,
    no_warning: bool,
) -> Result<(), EdenError>
```

### 2.3 Filtering Behavior

When `--no-warning` is active:

1. After collecting all findings, the function MUST filter:
   `findings.retain(|f| f.severity != "warning");`
2. The filter MUST apply before both human-mode and JSON-mode output.
3. The filter MUST apply before the `--strict` empty-check — only
   non-warning findings count toward the strict failure threshold.

### 2.4 Interaction with `--strict`

| `--strict` | `--no-warning` | Behavior |
| :--- | :--- | :--- |
| off | off | All findings shown, exit 0 |
| on | off | All findings shown, exit 3 if any findings |
| off | on | Only error + info findings shown, exit 0 |
| on | on | Only error + info findings shown, exit 3 if any remain |

### 2.5 Help Text

```text
  --no-warning    Hide warning-level findings from output
```

## 3. Summary Table Column Rename

### 3.1 Header

The summary table header MUST change from `Sev` to `Level`.

### 3.2 Cell Values

The severity cell function MUST return the full severity name:

| Internal Value | Displayed Value |
| :--- | :--- |
| `"error"` | `error` |
| `"warning"` | `warning` |
| `"info"` | `info` |

The previous abbreviation `warn` MUST NOT be used.

### 3.3 Column Width

The `ColumnConstraint::LowerBoundary` for the `Level` column MUST
change from `Width::Fixed(5)` to `Width::Fixed(7)` to accommodate
the full `warning` label (7 characters).

## 4. Severity Coloring

### 4.1 Color Rules

When `UiContext::colors_enabled()` is true, the `Level` cell MUST
be colored:

| Severity | Color | `owo_colors` Method |
| :--- | :--- | :--- |
| `error` | red | `.red()` |
| `warning` | yellow | `.yellow()` |
| `info` | dim (gray) | `.dimmed()` |

### 4.2 Implementation

The `doctor_severity_cell` function MUST accept a `&UiContext`
parameter and apply coloring conditionally:

```rust
fn doctor_severity_cell(ui: &UiContext, severity: &str) -> String {
    let label = match severity {
        "info" => "info",
        "warning" => "warning",
        _ => "error",
    };
    if !ui.colors_enabled() {
        return label.to_string();
    }
    match severity {
        "info" => label.dimmed().to_string(),
        "warning" => label.yellow().to_string(),
        _ => label.red().to_string(),
    }
}
```

### 4.3 Consistency with Detail Cards

The per-finding detail cards already use `doctor_severity_symbol`
which renders colored symbols (✗ red, ! yellow, · dim). The summary
table severity coloring MUST use the same color mapping for visual
consistency.

## 5. Example Output

### 5.1 Default (no `--no-warning`)

```text
  Doctor  4 issues detected

 ┌─────────┬────────────────────┬────────────────────────────────┐
 │ Level   ┆ Code               ┆ Skill                          │
 ╞═════════╪════════════════════╪════════════════════════════════╡
 │ error   ┆ TARGET_PATH_MISSING┆ vercel-react-best-practices    │
 │ warning ┆ LICENSE_UNKNOWN    ┆ vercel-react-best-practices    │
 └─────────┴────────────────────┴────────────────────────────────┘
```

(With colors: `error` in red, `warning` in yellow.)

### 5.2 With `--no-warning`

```text
  Doctor  1 issue detected

  ✗ [TARGET_PATH_MISSING] vercel-react-best-practices
    target path does not exist
    ~> Run `eden-skills apply` or `eden-skills repair` to recreate target paths.
```

(Summary table suppressed because only 1 finding remains, which is
≤ 3 — the existing threshold for table display.)

### 5.3 JSON Output with `--no-warning`

```json
{
  "summary": {
    "total": 1,
    "error": 1,
    "warning": 0
  },
  "findings": [
    {
      "code": "TARGET_PATH_MISSING",
      "severity": "error",
      "skill_id": "vercel-react-best-practices",
      "target_path": "...",
      "message": "target path does not exist",
      "remediation": "Run `eden-skills apply` or `eden-skills repair` to recreate target paths."
    }
  ]
}
```

## 6. Backward Compatibility

| Existing Feature | Phase 2.98 Behavior |
| :--- | :--- |
| `doctor` without `--no-warning` | Unchanged — all findings shown |
| `doctor --json` schema | Unchanged — `summary` and `findings` fields preserved |
| `doctor --strict` exit codes | Unchanged — exit 3 when findings exist (post-filter) |
| Detail card rendering | Unchanged — symbols and colors preserved |
| Summary table column count | Unchanged (3 columns) |
| Summary table column header | Changed: `Sev` → `Level` |
| Summary table cell values | Changed: `warn` → `warning` |
| Summary table cell styling | Changed: severity cells now colored |

## 7. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **DUX-001** | Builder | **P0** | `doctor` MUST accept a `--no-warning` flag. | `eden-skills doctor --no-warning` is a valid invocation. |
| **DUX-002** | Builder | **P0** | `--no-warning` MUST filter findings with `severity == "warning"` before output. | Doctor with `--no-warning` omits warning findings in human and JSON mode. |
| **DUX-003** | Builder | **P0** | `--no-warning` + `--strict` MUST only count non-warning findings for exit code. | `--strict --no-warning` exits 0 when only warnings exist. |
| **DUX-004** | Builder | **P0** | Summary table header MUST read `Level` instead of `Sev`. | Doctor summary table shows `Level` header. |
| **DUX-005** | Builder | **P0** | `Level` cell MUST display full severity name (`error`, `warning`, `info`). | No abbreviated `warn` appears in output. |
| **DUX-006** | Builder | **P0** | `Level` cell MUST be colored: red for `error`, yellow for `warning`, dim for `info`. | Summary table severity cells contain appropriate ANSI color sequences in TTY mode. |
