# SPEC_SCHEMA_P25.md

Phase 2.5 amendments to the `skills.toml` schema.

**Base contract:** `spec/phase1/SPEC_SCHEMA.md`
**Extended by:** `spec/phase2/SPEC_SCHEMA_EXT.md`
**Rule:** This file defines a minimal, targeted amendment to the Phase 1
schema. All other Phase 1 and Phase 2 schema rules remain in effect.

## 1. Purpose

Enable a zero-friction onboarding flow where users can start with an empty
config and progressively install skills via `eden-skills install <source>`.

## 2. Schema Amendment: Empty Skills Array

### 2.1 Current Rule (Phase 1)

From `spec/phase1/SPEC_SCHEMA.md` Section 2:

> `skills` MUST exist and contain at least one item

### 2.2 Amended Rule (Phase 2.5)

The `skills` array MAY be empty. A config file with `skills = []` or an
implicit empty `[[skills]]` section MUST be considered valid.

Rationale: the Phase 1 non-empty constraint was designed for a config-first
workflow where users manually author `skills.toml` before running commands.
Phase 2.5 introduces a command-first workflow (`install <url>`) that
progressively builds the config. Requiring a dummy skill entry creates
friction and produces misleading plan/apply output.

### 2.3 Behavioral Implications

| Command | Empty Skills Behavior |
| :--- | :--- |
| `plan` | Outputs empty plan (no actions). Exit code `0`. |
| `apply` | No source sync, no install mutations. Prints `apply summary: create=0 update=0 noop=0 conflict=0`. Exit code `0`. |
| `doctor` | No skill-level findings. Phase 2 registry/adapter findings still apply. Exit code `0`. |
| `repair` | No repair actions. Exit code `0`. |
| `list` | `list: 0 skill(s)`. Exit code `0`. |
| `install <source>` | Normal install flow; config transitions from empty to non-empty. |

## 3. Init Template Update

### 3.1 Current Template

The `init` command generates a config with a pre-populated `browser-tool`
skill entry pointing to `vercel-labs/skills.git`.

### 3.2 Updated Template

The `init` command MUST generate a minimal valid config with no skill entries:

```toml
version = 1

[storage]
root = "~/.local/share/eden-skills/repos"
```

This config is immediately usable with `eden-skills install <source>`.

### 3.3 Config Auto-Creation by Install

When `eden-skills install <source>` is invoked and the config file at the
resolved `--config` path does not exist:

- The CLI MUST auto-create the config file using the same minimal template
  as `init` (Section 3.2).
- The CLI MUST emit: `Created config at <path>`.
- The install command MUST then proceed normally, adding the installed skill
  to the newly created config.
- This auto-creation MUST NOT trigger if `--config` points to an explicitly
  non-existent path that the user appears to have mistyped (i.e., the parent
  directory does not exist). In that case, the CLI MUST fail with an IO error.

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **SCH-P25-001** | Builder | **P0** | `skills` array MAY be empty; empty config MUST pass validation. | Config with no `[[skills]]` entries loads without error. |
| **SCH-P25-002** | Builder | **P0** | `init` template MUST produce a minimal config without dummy skills. | `eden-skills init` creates config with `version = 1`, `[storage]`, no skills. |
| **SCH-P25-003** | Builder | **P0** | Phase 1 and Phase 2 configs with non-empty skills MUST remain valid. | Existing `skills.toml` with 5 skills loads and validates unchanged. |

## 5. Backward Compatibility

| Phase 1 Feature | Phase 2.5 Behavior |
| :--- | :--- |
| Configs with `[[skills]]` entries | Fully valid. No behavioral change. |
| `init --force` | Overwrites with new minimal template. |
| `add` command | Still works. Can add skills to an initially empty config. |
| Validation error codes | No new codes. The `EMPTY_SKILLS` error is removed (was implicit in Phase 1 validation). |
