# SPEC_REMOVE_ENH.md

Remove command enhancements and install convenience flags.

**Related contracts:**

- `spec/phase1/SPEC_COMMANDS.md` Section 3.6 (`remove`)
- `spec/phase2.5/SPEC_INSTALL_URL.md` Section 2 (`install` flags)

## 1. Purpose

Enhance the `remove` command with batch removal and interactive selection,
and add convenience flags to `install` for improved ergonomics. These are
quality-of-life improvements that reduce friction in daily workflows.

## 2. Batch Remove

### 2.1 Multiple Positional Arguments

The `remove` command MUST accept one or more skill IDs as positional
arguments:

```text
eden-skills remove <SKILL_ID>...
```

Examples:

```bash
# Remove a single skill (existing behavior, unchanged)
eden-skills remove browser-tool

# Remove multiple skills in one command
eden-skills remove browser-tool code-review filesystem-tool
```

Each skill ID MUST be validated against the config. If any ID is not
found, the CLI MUST report all unknown IDs in a single error and abort
without removing any skills (atomic validation).

Error format:

```text
error: unknown skill(s): 'nonexistent-a', 'nonexistent-b'
  → Available skills: browser-tool, code-review, filesystem-tool
```

### 2.2 Execution Order

When removing multiple skills, the CLI MUST:

1. Validate all IDs exist in config (fail-fast on any unknown ID).
2. For each skill (in the order provided):
   a. Invoke `uninstall_skill_targets()` (adapter uninstall + agent
      path cleanup + storage root cleanup).
   b. Remove the entry from the in-memory config.
3. Write the updated config once (single write, not per-skill).
4. Update the lock file once (single write).
5. Emit a summary.

### 2.3 Output

Human mode:

```text
  Remove   ✓ browser-tool
           ✓ code-review
           ✓ filesystem-tool

  ✓ 3 skills removed
```

JSON mode:

```json
{
  "action": "remove",
  "config_path": "~/.eden-skills/skills.toml",
  "removed": ["browser-tool", "code-review", "filesystem-tool"]
}
```

## 3. Interactive Remove

### 3.1 No-Argument Behavior

When `remove` is invoked with no positional arguments and stdout is a
TTY, the CLI MUST enter interactive mode:

1. Load the config and list all configured skills.
2. Display a numbered list with skill IDs and source info.
3. Prompt the user to select skills for removal.

```text
  Skills   3 configured:

    1. browser-tool       (vercel-labs/agent-skills)
    2. code-review        (user/code-review)
    3. filesystem-tool    (vercel-labs/agent-skills)

  Enter skill numbers or names to remove (space-separated):
  > 1 3

  Remove browser-tool, filesystem-tool? [y/N] y

  Remove   ✓ browser-tool
           ✓ filesystem-tool

  ✓ 2 skills removed
```

### 3.2 Non-TTY No-Argument Behavior

When `remove` is invoked with no positional arguments and stdout is NOT
a TTY, the CLI MUST fail with an error:

```text
error: no skill IDs specified
  → Usage: eden-skills remove <SKILL_ID>...
```

This prevents accidental mass removal in automation pipelines.

### 3.3 Confirmation Prompt

Interactive removal MUST display a confirmation prompt before executing.
The default answer MUST be `N` (no) to prevent accidental deletion.

The `-y` / `--yes` flag skips the confirmation prompt.

### 3.4 Empty Config

When the config contains no skills and `remove` is invoked without
arguments:

```text
  Skills   0 configured

  Nothing to remove.
```

Exit code `0`.

## 4. `--yes` Flag

### 4.1 `remove --yes`

The `-y` / `--yes` flag MUST skip the confirmation prompt in interactive
mode. When combined with positional arguments, it skips the confirmation
that would otherwise be shown for batch removal.

```bash
# Batch remove without confirmation
eden-skills remove browser-tool code-review -y
```

### 4.2 `install --yes`

The `-y` / `--yes` flag on `install` MUST skip all interactive
confirmation prompts (skill selection, install confirmation). This is
distinct from `--all`:

| Flag | Behavior |
| :--- | :--- |
| `--all` | Install all discovered skills (no prompt). |
| `--yes` | Accept the default action at each prompt (equivalent to pressing Enter). |
| `--all --yes` | Same as `--all` (no prompt to skip). |
| (neither) | Interactive mode with prompts. |

When `--yes` is used without `--all` on a multi-skill repo, the behavior
is equivalent to the user pressing `y` at each prompt → all skills are
installed.

## 5. `install --copy` Flag

The `install` command MUST accept a `--copy` flag that sets the install
mode to `copy` instead of the default `symlink`:

```bash
eden-skills install vercel-labs/agent-skills --copy
```

When `--copy` is provided, persisted config entries MUST use
`install.mode = "copy"` and the corresponding default verify checks
for copy mode (`["path-exists", "content-present"]`).

`--copy` is mutually exclusive with the existing `add --mode` flag
(which is not available on `install`). It provides a more discoverable
alternative for the common use case.

## 6. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **RMV-001** | Builder | **P1** | `remove` MUST accept multiple positional skill IDs. | `eden-skills remove a b c` removes all three skills. |
| **RMV-002** | Builder | **P1** | Unknown skill IDs in batch remove MUST be reported atomically without partial removal. | `remove known unknown` fails without removing `known`. |
| **RMV-003** | Builder | **P1** | `remove` with no arguments on TTY MUST enter interactive selection mode. | Interactive prompt lists skills and accepts selection. |
| **RMV-004** | Builder | **P1** | `remove` with no arguments on non-TTY MUST fail with error. | Piped `remove` without IDs exits with code 2. |
| **RMV-005** | Builder | **P1** | `-y` / `--yes` flag MUST skip confirmation prompts on `remove` and `install`. | `remove browser-tool -y` removes without prompting. |

## 7. Backward Compatibility

| Existing Feature | Phase 2.7 Behavior |
| :--- | :--- |
| `remove <single-id>` | Unchanged. Single positional argument still works. |
| `remove` JSON output | Extended to include `removed` array (additive). |
| `install --all` | Unchanged. `--yes` is a new complementary flag. |
| `add --mode` | Unchanged. `install --copy` is a separate convenience path. |
