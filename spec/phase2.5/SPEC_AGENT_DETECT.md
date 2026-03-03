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

The CLI MUST check for the existence of the following directories (relative
to the user's home directory) to determine which agents are installed.
Detection paths align with the conventions defined by the
[vercel-labs/skills](https://github.com/vercel-labs/skills) ecosystem.

| `--target` alias | Agent Name | Detection Path | Install Target |
| :--- | :--- | :--- | :--- |
| `claude-code` | Claude Code | `~/.claude/skills/` | `~/.claude/skills/` |
| `cursor` | Cursor | `~/.cursor/skills/` | `~/.cursor/skills/` |
| `antigravity` | Antigravity | `~/.gemini/antigravity/skills/` | `~/.gemini/antigravity/skills/` |
| `augment` | Augment | `~/.augment/skills/` | `~/.augment/skills/` |
| `openclaw` | OpenClaw | `~/.openclaw/skills/` | `~/.openclaw/skills/` |
| `cline` | Cline | `~/.agents/skills/` | `~/.agents/skills/` |
| `codebuddy` | CodeBuddy | `~/.codebuddy/skills/` | `~/.codebuddy/skills/` |
| `codex` | Codex | `~/.codex/skills/` | `~/.codex/skills/` |
| `command-code` | Command Code | `~/.commandcode/skills/` | `~/.commandcode/skills/` |
| `continue` | Continue | `~/.continue/skills/` | `~/.continue/skills/` |
| `cortex` | Cortex Code | `~/.snowflake/cortex/skills/` | `~/.snowflake/cortex/skills/` |
| `crush` | Crush | `~/.config/crush/skills/` | `~/.config/crush/skills/` |
| `droid` | Droid (Factory) | `~/.factory/skills/` | `~/.factory/skills/` |
| `gemini-cli` | Gemini CLI | `~/.gemini/skills/` | `~/.gemini/skills/` |
| `github-copilot` | GitHub Copilot | `~/.copilot/skills/` | `~/.copilot/skills/` |
| `goose` | Goose | `~/.config/goose/skills/` | `~/.config/goose/skills/` |
| `junie` | Junie | `~/.junie/skills/` | `~/.junie/skills/` |
| `iflow-cli` | iFlow CLI | `~/.iflow/skills/` | `~/.iflow/skills/` |
| `kilo` | Kilo Code | `~/.kilocode/skills/` | `~/.kilocode/skills/` |
| `kiro-cli` | Kiro CLI | `~/.kiro/skills/` | `~/.kiro/skills/` |
| `kode` | Kode | `~/.kode/skills/` | `~/.kode/skills/` |
| `mcpjam` | MCPJam | `~/.mcpjam/skills/` | `~/.mcpjam/skills/` |
| `mistral-vibe` | Mistral Vibe | `~/.vibe/skills/` | `~/.vibe/skills/` |
| `mux` | Mux | `~/.mux/skills/` | `~/.mux/skills/` |
| `opencode` | OpenCode | `~/.config/opencode/skills/` | `~/.config/opencode/skills/` |
| `openhands` | OpenHands | `~/.openhands/skills/` | `~/.openhands/skills/` |
| `pi` | Pi | `~/.pi/agent/skills/` | `~/.pi/agent/skills/` |
| `qoder` | Qoder | `~/.qoder/skills/` | `~/.qoder/skills/` |
| `qwen-code` | Qwen Code | `~/.qwen/skills/` | `~/.qwen/skills/` |
| `roo` | Roo | `~/.roo/skills/` | `~/.roo/skills/` |
| `trae` | Trae | `~/.trae/skills/` | `~/.trae/skills/` |
| `trae-cn` | Trae CN | `~/.trae-cn/skills/` | `~/.trae-cn/skills/` |
| `windsurf` | Windsurf | `~/.codeium/windsurf/skills/` | `~/.codeium/windsurf/skills/` |
| `zencoder` | Zencoder | `~/.zencoder/skills/` | `~/.zencoder/skills/` |
| `neovate` | Neovate | `~/.neovate/skills/` | `~/.neovate/skills/` |
| `pochi` | Pochi | `~/.pochi/skills/` | `~/.pochi/skills/` |
| `adal` | Adal | `~/.adal/skills/` | `~/.adal/skills/` |

### 2.2 Agents Sharing a Common Install Target

The following agents share the `~/.config/agents/skills/` install directory.
They are recognized as `--target` aliases but do NOT have independent
auto-detection rules (to avoid ambiguous identity for one shared path).
They are resolved when explicitly specified via `--target`:

| `--target` alias | Agent Name | Install Target |
| :--- | :--- | :--- |
| `amp` | Amp | `~/.config/agents/skills/` |
| `kimi-cli` | Kimi CLI | `~/.config/agents/skills/` |
| `replit` | Replit | `~/.config/agents/skills/` |
| `universal` | Universal | `~/.config/agents/skills/` |

### 2.3 Detection Logic

1. For each agent in the Section 2.1 table, check the detection path.
2. An agent is considered detected when either condition is true:
   - detection path exists and is a directory (`is_dir()`), or
   - detection path's parent directory exists and is a directory.
3. Collect all agents that satisfy the detection condition.
4. If one or more agents are detected → use them as install targets.
5. If no agents are detected → apply fallback behavior (Section 2.4).

The detection is implemented as a data-driven rule table
(`AGENT_RULES` in `crates/eden-skills-core/src/agents.rs`) to simplify
future additions. Detection order follows the table order in Section 2.1.

### 2.4 Fallback Behavior

When no known agent directory is detected:

- The CLI MUST default to `claude-code` as the sole target.
- The CLI MUST emit a warning: `No installed agents detected; defaulting to claude-code (~/.claude/skills/)`.

### 2.5 Target Directory Creation

For any resolved target (auto-detected or explicit), if the target skill
directory does not exist, the CLI MUST create it during install.

## 3. `--target` Override

When `--target` is explicitly provided on the command line:

- Auto-detection MUST be skipped entirely.
- Only the specified target(s) MUST be used.
- Multiple targets MAY be specified: `--target claude-code --target cursor`.
- The target spec format is the `--target` alias from Section 2.1 or 2.2,
  or `custom:<path>` for arbitrary paths.

## 4. Integration with Install Flow

### 4.1 Install Command

In URL-mode install (`SPEC_INSTALL_URL.md`), agent detection occurs at
step 6 of the execution pipeline (Section 6.1). The detected agents
are used to populate `[[skills.targets]]` entries in the persisted config.

### 4.2 Other Commands

Agent auto-detection is specific to the `install` command's URL mode.
Other commands (`add`, `apply`, `doctor`, `repair`) continue to use
the targets already defined in `skills.toml` and are not affected.

## 5. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **AGT-001** | Builder | **P0** | `install` without `--target` MUST auto-detect installed agents by checking detection paths from Section 2.1. | Install on a system with `~/.claude/skills/` and `~/.codeium/windsurf/skills/` detects both. |
| **AGT-002** | Builder | **P0** | Detection MUST check each agent's documented detection path (Section 2.1); agent is detected when `skills/` exists, or when only the parent config root exists. | Each detection path is checked; `skills/` or parent-root presence → included in targets. |
| **AGT-003** | Builder | **P0** | Explicit `--target` MUST override auto-detection entirely. | `--target cursor` installs to Cursor only, even if other agents are detected. |
| **AGT-004** | Builder | **P1** | No agents detected MUST fall back to `claude-code` with warning. | System with no agent dirs → install to `~/.claude/skills/` with warning. |
| **AGT-005** | Builder | **P1** | Detection implementation MUST be data-driven (`AGENT_RULES` table) to allow ecosystem expansion without code restructuring. | Adding a new agent requires only a new row in `AGENT_RULES` and `default_agent_path`. |
| **AGT-006** | Builder | **P1** | Agents sharing `~/.config/agents/skills/` (Section 2.2) MUST be installable via explicit `--target` but MUST NOT generate independent detection entries. | `--target kimi-cli` works; `~/.config/agents/skills/` does not auto-generate ambiguous `amp`/`kimi-cli`/`replit`/`universal` targets. |

## 6. Backward Compatibility

Detection and install defaults in this spec align to the
`Supported Agents` table's `Global Path` column from
`vercel-labs/skills`. Implementations that previously relied on
project-path-derived global defaults SHOULD migrate to these global paths.
