# SPEC_NEWLINE_POLICY.md

Newline normalization policy for all CLI output.

**Related contracts:**

- `spec/phase2.7/SPEC_OUTPUT_POLISH.md` (error format)
- `spec/phase2.8/SPEC_OUTPUT_UPGRADE.md` Section 5 (error hint `→`)

## 1. Problem Statement

Several commands produce inconsistent trailing whitespace:

1. `print_error()` in `main.rs` unconditionally outputs a blank line
   after the error message, even when no hint follows.
2. Clap parse errors contain built-in multi-line text (tip, usage,
   help reference) with trailing newlines. After passing through
   `print_error()`, the result has double trailing blank lines.
3. `remove` summary (`remove.rs:241`) emits a blank line after the
   final `✓ N skills removed` line.
4. Different commands have different trailing behavior — some end
   cleanly, others append spurious blank lines.

## 2. Normative Policy

### 2.1 Core Rule

**The last line of any command's output MUST NOT be followed by an
additional blank line.** The shell prompt MUST appear immediately
after the last content line.

### 2.2 Internal Section Spacing

| Scenario | Rule | Example |
| :--- | :--- | :--- |
| Between header and content | Exactly 1 blank line | `Found  4 skills:` → blank → cards |
| Between content sections | Exactly 1 blank line | Table → blank → summary |
| Between tree groups | No blank line | Group 1 last `└─` → next group `✓` |
| Between tree output and summary | Exactly 1 blank line | Last `└─` → blank → `✓ N installed` |
| After summary (end of output) | **No** blank line | `✓ N installed` → shell prompt |
| After table (end of output) | **No** blank line | Table → shell prompt |
| After table (more output follows) | Exactly 1 blank line | Table → blank → summary |

### 2.3 Error Output Spacing

| Scenario | Rule |
| :--- | :--- |
| Error with hint | `error: msg` → blank → `  → hint` → (end, no trailing blank) | <!-- markdownlint-disable-line -->
| Error without hint | `error: msg` → (end, no trailing blank) |
| Multi-line error message | Print as-is; no additional blank line |

## 3. Implementation Fixes

### 3.1 `print_error()` in `main.rs`

**Before:**

```rust
eprintln!("{prefix} {message}");
eprintln!();
if let Some(hint) = hint {
    // ...
    eprintln!("  {} {hint}", "→".dimmed());
}
```

**After:**

```rust
eprintln!("{prefix} {message}");
if let Some(hint) = hint {
    eprintln!();
    if colors_enabled {
        eprintln!("  {} {hint}", "→".dimmed());
    } else {
        eprintln!("  → {hint}");
    }
}
```

The blank line is moved to only appear as a separator before the
hint. When no hint exists, the output ends immediately after the
error message.

### 3.2 Clap Error Trimming in `lib.rs`

**Before:**

```rust
let msg = raw.strip_prefix("error: ").unwrap_or(&raw).to_string();
```

**After:**

```rust
let msg = raw.strip_prefix("error: ").unwrap_or(&raw)
    .trim_end()
    .to_string();
```

Clap error strings contain trailing `\n` characters from their
built-in formatting. `.trim_end()` removes these before the string
is wrapped in `EdenError::InvalidArguments` and later processed by
`print_error()`.

### 3.3 `remove` Summary

Remove the `println!()` on line 241 of `remove.rs`:

**Before:**

```rust
println!("{}  {} {}", ui.action_prefix("Remove"), success, removed[0]);
for skill_id in removed.iter().skip(1) {
    println!("          {} {}", success, skill_id);
}
println!();  // ← remove this
let noun = ...;
println!("  {} {} {} removed", success, removed.len(), noun);
```

**After:**

```rust
println!("{}  {} {}", ui.action_prefix("Remove"), success, removed[0]);
for skill_id in removed.iter().skip(1) {
    println!("          {} {}", success, skill_id);
}
println!();
println!("  {} {} {} removed", success, removed.len(), noun);
```

Wait — the blank line between the per-skill lines and the summary
is intentional (Section 2.2: between content sections). The issue
is actually that `print_remove_summary` ends with the summary line
and the caller doesn't add more blank lines. Let me re-examine...

The actual fix: ensure that `print_remove_summary` ends with the
summary line and nothing else. The blank line between the per-skill
list and the summary (`println!()` on what is currently line 241)
is correct — it separates content sections. The problem is if
there's an additional blank line AFTER the summary. In the current
code, the summary line is the last thing printed, and no extra
blank line follows. So the actual trailing-newline issue in `remove`
may be a misdiagnosis from Phase 2.9 planning.

**Revised fix:** Audit `remove.rs` and confirm the summary is the
last output. If the blank line is between sections (correct), no
change is needed. The Builder MUST verify this during implementation.

### 3.4 Systematic Audit

The Builder MUST audit ALL human-mode output paths and ensure:

1. No `println!()` (empty) appears as the last statement before
   `Ok(())` in any command function.
2. No `eprintln!()` (empty) appears as the last statement in error
   display.
3. Section-separating blank lines appear only BETWEEN sections, never
   after the final section.

### 3.5 Affected Files

| File | Known Issue | Fix |
| :--- | :--- | :--- |
| `main.rs` | Unconditional blank line in `print_error` | Move blank line inside `if hint` |
| `lib.rs` | Clap error trailing `\n` | `.trim_end()` |
| `remove.rs` | Audit trailing output | Verify no trailing blank line |
| `update.rs` | Audit trailing output after summary/failure lines | Verify no trailing blank line |
| `reconcile.rs` | Audit trailing output after verification line | Verify no trailing blank line |
| `config_ops.rs` | Audit `init`, `list`, `add`, `set`, `config_export`, `config_import` | Remove any trailing blank lines |
| `diagnose.rs` | Audit trailing output after finding cards | Verify no trailing blank line |
| `plan_cmd.rs` | Audit trailing output | Verify no trailing blank line |
| `install.rs` | Audit trailing output after summary | Verify no trailing blank line |

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **NLP-001** | Builder | **P0** | The last line of command output MUST NOT be followed by a blank line. | All commands tested for trailing blank line absence. |
| **NLP-002** | Builder | **P0** | `print_error` MUST only emit a blank line when a hint follows. | Error without hint ends immediately; error with hint has blank separator. |
| **NLP-003** | Builder | **P0** | Clap error strings MUST be `.trim_end()`'d before wrapping in `EdenError`. | Clap errors display without trailing whitespace. |
| **NLP-004** | Builder | **P0** | Section spacing MUST follow Section 2.2 rules. | Between sections: 1 blank line. End of output: no blank line. |
| **NLP-005** | Builder | **P1** | Builder MUST audit all command output paths per Section 3.4. | Audit complete; no trailing blank lines in any command. |
| **NLP-006** | Builder | **P1** | All `println!()` and `eprintln!()` calls at end-of-function MUST be verified against the policy. | No trailing empty `println!()` before `Ok(())`. |

## 5. Backward Compatibility

| Existing Feature | Phase 2.9 Behavior |
| :--- | :--- |
| `--json` output | Unchanged. JSON formatting is not affected by newline policy. |
| Exit codes | Unchanged. |
| Error hint `→` | Format preserved; only spacing adjusted. |
| Non-TTY output | Same newline policy applies. |
