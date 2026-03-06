# SPEC_HINT_SYNC.md

Hint arrow prefix synchronization: `→` (dimmed) → `~>` (magenta).

**Related contracts:**

- `spec/phase2.8/SPEC_OUTPUT_UPGRADE.md` Section 5 (OUP-013)
- `spec/phase2.9/SPEC_NEWLINE_POLICY.md` Section 4

## 1. Problem Statement

Phase 2.8 `SPEC_OUTPUT_UPGRADE.md` (OUP-013) and Phase 2.9
`SPEC_NEWLINE_POLICY.md` define the error hint prefix as `→`
rendered in dimmed style. However, the shipped implementation
uses `~>` rendered in **magenta** — a deliberate choice because:

1. `~>` renders correctly in all terminal emulators (no Unicode
   rendering risk of `→` being squeezed or misaligned).
2. Magenta provides higher visual contrast than dimmed gray,
   making hints more noticeable.

This spec formally amends the earlier contracts to match the
shipped implementation.

## 2. Amendment

### 2.1 Hint Prefix Character

All hint/guidance/remediation lines MUST use `~>` as the prefix
character, not `→`.

### 2.2 Hint Prefix Style

When colors are enabled, `~>` MUST be rendered in **magenta**.
When colors are disabled, `~>` is rendered as plain text.

### 2.3 Affected Locations

| Location | Before (spec) | After (amended) |
| :--- | :--- | :--- |
| Error hint in `main.rs` | `→` dimmed | `~>` magenta |
| Doctor remediation | `→` dimmed | `~>` magenta |
| Update guidance | `→ Run ...` | `~> Run ...` |
| Plan action hints | `→` dimmed | `~>` magenta |
| Any future hint output | `→` dimmed | `~>` magenta |

### 2.4 Code Pattern

```rust
if colors_enabled {
    eprintln!("  {} {hint}", "~>".magenta());
} else {
    eprintln!("  ~> {hint}");
}
```

## 3. Spec Files Amended

| File | Section | Change |
| :--- | :--- | :--- |
| `phase2.8/SPEC_OUTPUT_UPGRADE.md` | Section 5.1, 5.2, 5.3 | `→` (dimmed) → `~>` (magenta) |
| `phase2.8/SPEC_OUTPUT_UPGRADE.md` | OUP-013 requirement | Updated prefix |
| `phase2.9/SPEC_NEWLINE_POLICY.md` | Section 4, error format | `→` (dimmed) → `~>` (magenta) |
| `phase2.8/SPEC_TEST_MATRIX.md` | TM-P28-026 | Updated expected prefix |

Note: The frozen spec files are NOT physically modified. This
amendment document serves as the authoritative override per the
authority chain (`spec/` > `STATUS.yaml` > ...).

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **HSY-001** | Builder | **P1** | All hint/guidance/remediation lines MUST use `~>` (not `→`) as prefix. | No occurrence of `→` as hint prefix in CLI output code. |
| **HSY-002** | Builder | **P1** | `~>` MUST be styled magenta when colors are enabled. | Hint prefix contains ANSI magenta escape code. |
