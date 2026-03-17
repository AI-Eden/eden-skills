# SPEC_VERIFY_DEDUP.md

Short-circuit dependent verify checks when the target path is missing.

**Related contracts:**

- `spec/phase1/SPEC_COMMANDS.md` (doctor/repair verification)
- `spec/phase1/SPEC_SCHEMA.md` (`verify.checks` configuration)

## 1. Problem Statement

When a symlink target is removed (e.g., via `unlink`), the three
default verify checks (`path-exists`, `is-symlink`, `target-resolves`)
each independently detect the absence and produce separate findings:

| Check | Failure | Finding Code |
| :--- | :--- | :--- |
| `path-exists` | `symlink_metadata` returns `Err` | `TARGET_PATH_MISSING` |
| `is-symlink` | `symlink_metadata` returns `Err` | `BROKEN_SYMLINK` |
| `target-resolves` | `read_link` returns `Err` | `BROKEN_SYMLINK` |

All three describe the same root cause — the target path does not
exist. The two `BROKEN_SYMLINK` findings are noise because they add
no actionable information beyond `TARGET_PATH_MISSING`.

This redundancy compounds in `doctor` output: a single broken skill
produces 3 error-level findings plus any safety warnings, making the
output harder to scan.

## 2. Root Cause Analysis

The `is-symlink` and `target-resolves` checks implicitly depend on
the target path existing on the filesystem:

- `is-symlink` calls `fs::symlink_metadata(target_path)` — if the
  path does not exist, this returns `Err`, which falls into the
  `BROKEN_SYMLINK` branch (not the `TARGET_NOT_SYMLINK` branch).
- `target-resolves` calls `fs::read_link(target_path)` — if the path
  does not exist, this returns `Err`, producing "target symlink is
  missing or unreadable".

Both checks produce **strictly less informative** findings than
`path-exists` when the root cause is path absence.

## 3. Solution: Pre-Check Short-Circuit

### 3.1 Behavior

In `verify_config_state()`, before entering the per-check loop for
each `(skill, target)` pair, the function MUST probe whether the
target path exists using `fs::symlink_metadata`. If the target does
not exist:

- The `path-exists` check (if present in `verify.checks`) MUST still
  run and produce its `TARGET_PATH_MISSING` finding.
- All other checks (`is-symlink`, `target-resolves`, `content-present`)
  MUST be skipped for that target.

### 3.2 Implementation

```rust
for target in &skill.targets {
    let target_root = resolve_target_path(target, config_dir)?;
    let target_path = normalize_lexical(&target_root.join(&skill.id));
    let target_exists = fs::symlink_metadata(&target_path).is_ok();

    for check in &skill.verify.checks {
        if !target_exists && check != "path-exists" {
            continue;
        }
        run_check(
            check,
            skill.id.as_str(),
            skill.install.mode,
            &source_path,
            &target_path,
            &mut issues,
        )?;
    }
}
```

### 3.3 Semantics Table

| Target State | `path-exists` | `is-symlink` | `target-resolves` | `content-present` |
| :--- | :--- | :--- | :--- | :--- |
| Missing | runs → `TARGET_PATH_MISSING` | **skipped** | **skipped** | **skipped** |
| Exists, not symlink | passes | runs → `TARGET_NOT_SYMLINK` | runs (may fail) | runs |
| Exists, symlink, broken target | passes | passes | runs → `BROKEN_SYMLINK` | runs |
| Exists, symlink, wrong target | passes | passes | runs → `TARGET_RESOLVE_MISMATCH` | runs |
| Exists, symlink, correct target | passes | passes | passes | runs |

### 3.4 Edge Case: `path-exists` Not in Check List

If a skill's `verify.checks` does not include `path-exists` but the
target is missing, the short-circuit still applies: dependent checks
are skipped. No finding is produced for that target. This is correct
because the user explicitly opted out of the existence check.

## 4. Impact on `repair`

The `repair` command calls `verify_config_state()` to discover issues
and then remediates them. With this change:

- **Before:** `repair` receives 3 findings for a missing target
  (`TARGET_PATH_MISSING` + 2× `BROKEN_SYMLINK`).
- **After:** `repair` receives 1 finding (`TARGET_PATH_MISSING`).

The repair logic remediates by recreating the symlink/copy based on
the skill's `install.mode`. It does not require multiple findings to
trigger remediation — a single `TARGET_PATH_MISSING` is sufficient.
Therefore, **repair behavior is unchanged**.

## 5. Impact on `doctor`

With the verify dedup in place:

- A single missing symlink produces 1 error finding instead of 3.
- The `doctor` summary count accurately reflects the number of
  distinct issues.
- Combined with `SPEC_DOCTOR_UX.md` `--no-warning`, users get a
  clean, non-redundant diagnostic view.

## 6. Performance

The short-circuit eliminates 2 redundant `symlink_metadata` /
`read_link` syscalls per missing target. For a config with N skills
and M targets per skill, the worst-case reduction is `2 × N × M`
syscalls. The performance impact is negligible but directionally
positive.

## 7. Backward Compatibility

| Existing Feature | Phase 2.98 Behavior |
| :--- | :--- |
| `doctor` finding codes | Unchanged — same codes, fewer duplicates |
| `doctor --json` schema | Unchanged — fewer findings in array |
| `repair` remediation | Unchanged — repair actions keyed on skill/target, not finding count |
| `verify.checks` configuration | Unchanged — no schema change |
| Custom check lists | Supported — short-circuit respects per-skill check list |

## 8. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **VDD-001** | Builder | **P0** | When `target_path` does not exist, `verify_config_state` MUST skip all checks except `path-exists`. | Missing target produces exactly 1 `TARGET_PATH_MISSING` finding (not 3). |
| **VDD-002** | Builder | **P0** | When `target_path` exists, all configured checks MUST run normally. | Existing symlink with wrong target still produces `TARGET_RESOLVE_MISMATCH`. |
| **VDD-003** | Builder | **P1** | `repair` MUST remediate correctly with the reduced finding set. | `repair` after `unlink` restores the symlink with a single finding. |
