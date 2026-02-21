# SPEC_LOCK.md

Lock file specification for deterministic state reconciliation.

**Related contracts:**

- `spec/phase1/SPEC_SCHEMA.md` (config schema)
- `spec/phase1/SPEC_COMMANDS.md` Section 3 (`plan`, `apply`)
- `spec/phase2/SPEC_REACTOR.md` (async reconciliation)

## 1. Purpose

Introduce a `skills.lock` file that records the **resolved installed state**
after each successful reconciliation. The lock file enables:

1. **Diff-driven reconciliation** — `apply` can detect skills removed from
   `skills.toml` and generate `Remove` actions to clean up orphan files.
2. **Exact reproducibility** — lock captures resolved commit SHAs and
   registry versions so collaborators reproduce the same state.
3. **Faster re-runs** — unchanged skills can be skipped during source sync
   by comparing lock entries against current TOML entries.

## 2. File Location

### 2.1 Default

The lock file MUST be placed adjacent to `skills.toml`:

```text
~/.eden-skills/skills.toml      ← config (source of truth for intent)
~/.eden-skills/skills.lock      ← lock  (record of installed reality)
```

### 2.2 Custom Config Path

When `--config /path/to/my-skills.toml` is used, the lock file MUST be
co-located:

```text
/path/to/my-skills.toml
/path/to/my-skills.lock
```

The lock path is derived by replacing the `.toml` extension with `.lock`.
If the config file has no `.toml` extension, `.lock` is appended.

### 2.3 Lock File Ownership

The lock file is fully managed by the CLI. Users SHOULD NOT edit it manually.
If the lock file is missing or corrupted, the CLI MUST fall back to
full reconciliation (equivalent to a clean `apply`) and regenerate the
lock file from scratch.

## 3. File Format

The lock file MUST use TOML syntax, consistent with `skills.toml` and
`Cargo.lock`.

### 3.1 Top-Level Structure

```toml
version = 1

[[skills]]
id = "browser-tool"
source_repo = "https://github.com/vercel-labs/agent-skills.git"
source_subpath = "skills/browser-tool"
source_ref = "main"
resolved_commit = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
install_mode = "symlink"
installed_at = "2026-02-21T10:30:00Z"

[[skills.targets]]
agent = "claude-code"
path = "~/.claude/skills/browser-tool"

[[skills.targets]]
agent = "cursor"
path = "~/.cursor/skills/browser-tool"

[[skills]]
id = "code-review"
source_repo = "registry://official/code-review"
source_subpath = "."
source_ref = "main"
resolved_commit = ""
resolved_version = "1.2.0"
install_mode = "symlink"
installed_at = "2026-02-21T10:30:05Z"

[[skills.targets]]
agent = "claude-code"
path = "~/.claude/skills/code-review"
```

### 3.2 Field Definitions

| Field | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `version` | integer | MUST | Lock file format version. Currently `1`. |
| `skills[].id` | string | MUST | Skill identifier, matches `skills.toml` entry. |
| `skills[].source_repo` | string | MUST | Resolved source URL or `registry://` URI. |
| `skills[].source_subpath` | string | MUST | Subpath within repo. `"."` for root. |
| `skills[].source_ref` | string | MUST | Branch, tag, or commit ref used. |
| `skills[].resolved_commit` | string | SHOULD | Full SHA-1 of the installed commit. Empty string when unavailable (e.g., local path source). |
| `skills[].resolved_version` | string | MAY | Resolved semver version for registry-mode skills. Absent for URL-mode. |
| `skills[].install_mode` | string | MUST | `"symlink"` or `"copy"`. |
| `skills[].installed_at` | string | MUST | ISO 8601 UTC timestamp of last install/update. |
| `skills[].targets` | array | MUST | Array of installed target records. |
| `skills[].targets[].agent` | string | MUST | Agent identifier (e.g., `"claude-code"`, `"cursor"`, `"custom"`). |
| `skills[].targets[].path` | string | MUST | Resolved target path where skill was installed. |

### 3.3 Ordering

Skills MUST be serialized in alphabetical order by `id` for deterministic
diffs. Targets within a skill MUST be serialized in alphabetical order by
`agent`.

### 3.4 Version Field

The top-level `version` field MUST be `1` for this specification. Future
lock format changes MUST increment the version. The CLI MUST reject lock
files with unknown versions by falling back to full reconciliation.

## 4. Lock File Lifecycle

### 4.1 Creation

The lock file is created or updated after any command that mutates
installed state:

| Command | Lock Action |
| :--- | :--- |
| `apply` | Write lock with all installed skills after reconciliation. |
| `repair` | Write lock after repair completes. |
| `install` | Write lock after skill is installed. |
| `remove` | Write lock after skill is uninstalled. |
| `update` | Write lock after registry sources are updated. |
| `init` | Write empty lock (no skills). |
| `plan` | Read-only. Lock is NOT modified. |
| `doctor` | Read-only. Lock is NOT modified. |
| `list` | Read-only. Lock is NOT modified. |

### 4.2 Reading

The lock file is read at the start of `apply`, `repair`, and `plan` to
enable diff-driven reconciliation.

### 4.3 Missing Lock File

When the lock file does not exist:

- The CLI MUST proceed as if no skills were previously installed.
- After successful execution of a mutating command, the lock file MUST be
  created.
- The CLI MUST NOT emit an error or warning for a missing lock file (this
  is the normal state on first run and during migration from pre-2.7).

### 4.4 Corrupted Lock File

When the lock file exists but fails TOML parsing or schema validation:

- The CLI MUST emit a warning: `warning: skills.lock is corrupted; performing full reconciliation`.
- The CLI MUST proceed with full reconciliation (no diff optimization).
- After successful execution, the lock file MUST be regenerated.

### 4.5 Lock File and Git

The lock file SHOULD be committed to version control when `skills.toml` is
shared across a team, analogous to `Cargo.lock`. This enables:

- Reproducible builds across team members.
- `git diff` visibility into skill state changes.

## 5. Diff-Driven Reconciliation

### 5.1 Apply Diff Algorithm

When `apply` runs with a valid lock file present, the CLI MUST compute a
three-way diff:

```text
Input:
  T = skills declared in skills.toml  (desired state)
  L = skills recorded in skills.lock  (last-known installed state)

Diff:
  ADDED   = { s ∈ T | s.id ∉ L }           → Create actions
  REMOVED = { s ∈ L | s.id ∉ T }           → Remove actions
  CHANGED = { s ∈ T ∩ L | s differs }      → Update actions
  UNCHANGED = { s ∈ T ∩ L | s identical }   → Noop (may skip source sync)
```

### 5.2 Remove Action Semantics

For each skill in the `REMOVED` set, the CLI MUST:

1. Resolve target paths from the lock entry's `targets` array.
2. For each target, invoke `adapter.uninstall()` to remove the installed
   path (symlink or directory).
3. Scan known default agent paths for leftover entries matching `skill.id`
   and remove them (same cleanup as the `remove` command).
4. Remove the canonical storage directory at `<storage.root>/<skill.id>`.
5. Emit a status line: `Remove  ✓ <skill_id>` (human mode) or include
   the removal in the JSON output array.

### 5.3 Change Detection

A skill is considered `CHANGED` (requiring Update) when any of these
fields differ between the lock entry and the TOML entry:

- `source.repo`
- `source.subpath`
- `source.ref`
- `install.mode`
- `targets` (set of agent+path pairs)

A skill whose only difference is `resolved_commit` (i.e., upstream moved
forward) is classified as `CHANGED` during `apply` if a source sync
fetches new content.

### 5.4 Noop Optimization

A skill in the `UNCHANGED` set MAY skip source sync (`git fetch`) to
accelerate `apply`. The CLI SHOULD verify that the local storage
directory still exists before skipping. If the storage directory is
missing despite being in the lock, the skill MUST be reclassified as
`CHANGED`.

### 5.5 Plan Integration

The `plan` command MUST also use the lock file for diff computation.
The `Action` enum MUST be extended with a `Remove` variant:

```rust
pub enum Action {
    Create,
    Update,
    Noop,
    Conflict,
    Remove,   // NEW in Phase 2.7
}
```

`plan` output for `Remove` actions:

```text
  Plan     5 actions

  create   new-skill → ~/.claude/skills/new-skill (symlink)
  remove   old-skill → ~/.claude/skills/old-skill
  remove   old-skill → ~/.cursor/skills/old-skill
  noop     stable-skill → ~/.claude/skills/stable-skill
```

### 5.6 Strict Mode Interaction

In `--strict` mode, `Remove` actions are NOT treated as conflicts. A
skill intentionally removed from `skills.toml` is a legitimate state
transition. Strict mode MUST still flag conflicts (e.g., target-not-symlink)
but MUST NOT block removals.

### 5.7 JSON Mode Integration

When `--json` is set, `apply` MUST include removal actions in the output:

```json
{
  "action": "remove",
  "skill_id": "old-skill",
  "target_path": "~/.claude/skills/old-skill",
  "status": "success"
}
```

### 5.8 Reactor Integration

Removal actions during `apply` MUST participate in the async reactor's
bounded concurrency, alongside create and update actions. Removal
operations MUST use the same `TargetAdapter` interface
(`adapter.uninstall()`).

## 6. Backward Compatibility

| Scenario | Behavior |
| :--- | :--- |
| Pre-2.7 installation (no lock file) | `apply` performs full reconciliation; lock is created afterward. |
| Downgrade from 2.7 to older version | Older CLI ignores `skills.lock` (unrecognized file). |
| Lock version mismatch | CLI warns and performs full reconciliation. |

## 7. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **LCK-001** | Builder | **P0** | `apply` MUST generate `Remove` actions for skills present in lock but absent from TOML. | Skill removed from TOML is cleaned up after `apply`. |
| **LCK-002** | Builder | **P0** | Lock file MUST be written after every mutating command (`apply`, `repair`, `install`, `remove`, `update`, `init`). | Lock file exists and reflects current installed state after each command. |
| **LCK-003** | Builder | **P0** | Lock file MUST use TOML format with fields defined in Section 3.2. | Lock file parses as valid TOML; all required fields present. |
| **LCK-004** | Builder | **P0** | Lock file MUST be co-located with the config file (Section 2.2). | `--config /tmp/my.toml` produces `/tmp/my.lock`. |
| **LCK-005** | Builder | **P0** | Missing lock file MUST NOT cause errors; CLI falls back to full reconciliation. | First-ever `apply` succeeds and creates lock. |
| **LCK-006** | Builder | **P0** | Corrupted lock file MUST emit warning and proceed with full reconciliation. | Garbage lock content triggers warning; apply still succeeds. |
| **LCK-007** | Builder | **P0** | `plan` MUST show `Remove` actions when lock contains skills absent from TOML. | `plan` output includes `remove` lines for orphaned skills. |
| **LCK-008** | Builder | **P1** | Unchanged skills MAY skip source sync when lock entry matches TOML entry and storage directory exists. | Re-running `apply` with no config changes is faster than first run. |
| **LCK-009** | Builder | **P0** | Lock entries MUST be sorted alphabetically by `id` for deterministic serialization. | Lock file diffs are minimal and stable. |
| **LCK-010** | Builder | **P1** | `resolved_commit` SHOULD record the full SHA-1 of the installed commit. | Lock entry contains 40-character hex SHA after `apply`. |

## 8. Future Scope (Not in Phase 2.7)

- Lock-based `apply --dry-run` diff summary (beyond `plan` output).
- Lock-based integrity verification (compare `resolved_commit` against
  on-disk content hash).
- Lock file merge conflict resolution tooling.
