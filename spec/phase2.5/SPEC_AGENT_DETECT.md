# SPEC_AGENT_DETECT.md

Agent auto-detection for install target resolution.

**Related Phase 1 contract:** `spec/phase1/SPEC_AGENT_PATHS.md`
**Rule:** This spec extends the agent path resolution policy with active
detection capabilities. Phase 1 path resolution rules remain in effect.

## 1. Purpose

When a user runs `eden-skills install <source>` without specifying `--target`,
the CLI should automatically detect which coding agents are installed on the
system and install the skill to all detected agent skill directories.

## 2. Detection Strategy

### 2.1 Supported Agents and Detection Paths

The CLI MUST check for the existence of the following directories to
determine which agents are installed:

| Agent | `--target` value | Detection Path (Global) | Install Target |
| :--- | :--- | :--- | :--- |
| Claude Code | `claude-code` | `~/.claude/` | `~/.claude/skills/` |
| Cursor | `cursor` | `~/.cursor/` | `~/.cursor/skills/` |
| Codex | `codex` | `~/.codex/` | `~/.codex/skills/` |
| Windsurf | `windsurf` | `~/.codeium/windsurf/` | `~/.codeium/windsurf/skills/` |

This list MAY be expanded in future releases. The detection implementation
SHOULD use a data-driven lookup table to simplify additions.

### 2.2 Detection Logic

1. For each agent in the table, check if the detection path exists and is
   a directory.
2. Collect all agents whose detection path exists.
3. If one or more agents are detected → use them as install targets.
4. If no agents are detected → apply fallback behavior (Section 2.3).

### 2.3 Fallback Behavior

When no known agent directory is detected:

- The CLI MUST default to `claude-code` as the sole target.
- The CLI MUST emit a warning: `No installed agents detected; defaulting to claude-code (~/.claude/skills/)`.

### 2.4 Target Directory Creation

If the agent's skill directory (e.g., `~/.claude/skills/`) does not exist
but the agent root directory (e.g., `~/.claude/`) does exist, the CLI
MUST create the skill directory during install.

## 3. `--target` Override

When `--target` is explicitly provided on the command line:

- Auto-detection MUST be skipped entirely.
- Only the specified target(s) MUST be used.
- Multiple targets MAY be specified: `--target claude-code --target cursor`.
- The target spec format follows the existing Phase 1 convention
  (`claude-code`, `cursor`, `custom:<path>`).

## 4. Integration with Install Flow

### 4.1 Install Command

In URL-mode install (`SPEC_INSTALL_URL.md`), agent detection occurs at
step 6 of the execution pipeline (Section 6.1). The detected agents
are used to populate `[[skills.targets]]` entries in the persisted
config.

### 4.2 Other Commands

Agent auto-detection is specific to the `install` command's URL mode.
Other commands (`add`, `apply`, `doctor`, `repair`) continue to use
the targets already defined in `skills.toml` and are not affected.

## 5. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **AGT-001** | Builder | **P0** | `install` without `--target` MUST auto-detect installed agents. | Install on a system with `~/.claude/` and `~/.cursor/` installs to both. |
| **AGT-002** | Builder | **P0** | Detection MUST check documented agent directories (Section 2.1 table). | Each agent directory is checked; presence → included in targets. |
| **AGT-003** | Builder | **P0** | Explicit `--target` MUST override auto-detection. | `--target cursor` installs to Cursor only, even if Claude Code is detected. |
| **AGT-004** | Builder | **P1** | No agents detected MUST fall back to claude-code with warning. | System with no agent dirs → install to `~/.claude/skills/` with warning. |

## 6. Future Scope (Not in Phase 2.5)

- Project-level agent detection (`.claude/skills/` in cwd).
- `--global` vs project scope distinction.
- Agent-specific skill compatibility checks.
- Broader agent support (40+ agents from `vercel-labs/skills` ecosystem).
