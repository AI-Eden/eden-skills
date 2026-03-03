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
| `claude-code` | Claude Code | `~/.claude/` | `~/.claude/skills/` |
| `cursor` | Cursor | `~/.agents/` | `~/.agents/skills/` |
| `antigravity` | Antigravity | `~/.agent/` | `~/.agent/skills/` |
| `augment` | Augment | `~/.augment/` | `~/.augment/skills/` |
| `openclaw` | Openclaw | `~/skills/` | `~/skills/` |
| `cline` | Cline | `~/.cline/` | `~/.cline/skills/` |
| `codebuddy` | CodeBuddy | `~/.codebuddy/` | `~/.codebuddy/skills/` |
| `command-code` | Command Code | `~/.commandcode/` | `~/.commandcode/skills/` |
| `continue` | Continue | `~/.continue/` | `~/.continue/skills/` |
| `cortex` | Cortex | `~/.cortex/` | `~/.cortex/skills/` |
| `crush` | Crush | `~/.crush/` | `~/.crush/skills/` |
| `droid` | Droid (Factory) | `~/.factory/` | `~/.factory/skills/` |
| `goose` | Goose | `~/.goose/` | `~/.goose/skills/` |
| `junie` | Junie | `~/.junie/` | `~/.junie/skills/` |
| `iflow-cli` | iFlow CLI | `~/.iflow/` | `~/.iflow/skills/` |
| `kilo` | Kilo Code | `~/.kilocode/` | `~/.kilocode/skills/` |
| `kiro-cli` | Kiro CLI | `~/.kiro/` | `~/.kiro/skills/` |
| `kode` | Kode | `~/.kode/` | `~/.kode/skills/` |
| `mcpjam` | MCPJam | `~/.mcpjam/` | `~/.mcpjam/skills/` |
| `mistral-vibe` | Mistral Vibe | `~/.vibe/` | `~/.vibe/skills/` |
| `mux` | Mux | `~/.mux/` | `~/.mux/skills/` |
| `openhands` | OpenHands | `~/.openhands/` | `~/.openhands/skills/` |
| `pi` | Pi | `~/.pi/` | `~/.pi/skills/` |
| `qoder` | Qoder | `~/.qoder/` | `~/.qoder/skills/` |
| `qwen-code` | Qwen Code | `~/.qwen/` | `~/.qwen/skills/` |
| `roo` | Roo | `~/.roo/` | `~/.roo/skills/` |
| `trae` | Trae | `~/.trae/` | `~/.trae/skills/` |
| `windsurf` | Windsurf | `~/.windsurf/` | `~/.windsurf/skills/` |
| `zencoder` | Zencoder | `~/.zencoder/` | `~/.zencoder/skills/` |
| `neovate` | Neovate | `~/.neovate/` | `~/.neovate/skills/` |
| `pochi` | Pochi | `~/.pochi/` | `~/.pochi/skills/` |
| `adal` | Adal | `~/.adal/` | `~/.adal/skills/` |

### 2.2 Agents Sharing a Common Install Target

The following agents share the `~/.agents/skills/` install directory
(the same as `cursor`) and therefore share the same detection signal.
They are recognized as `--target` aliases but do NOT have independent
detection rules — they are resolved only when explicitly specified via
`--target`:

| `--target` alias | Agent Name | Install Target |
| :--- | :--- | :--- |
| `amp` | Amp | `~/.agents/skills/` |
| `codex` | Codex | `~/.agents/skills/` |
| `gemini-cli` | Gemini CLI | `~/.agents/skills/` |
| `github-copilot` | GitHub Copilot | `~/.agents/skills/` |
| `kimi-cli` | Kimi CLI | `~/.agents/skills/` |
| `opencode` | Opencode | `~/.agents/skills/` |
| `replit` | Replit | `~/.agents/skills/` |
| `universal` | Universal | `~/.agents/skills/` |
| `trae-cn` | Trae CN | `~/.trae/skills/` |

### 2.3 Detection Logic

1. For each agent in the Section 2.1 table, check if the detection path exists
   and is a directory (`is_dir()`).
2. Collect all agents whose detection path exists.
3. If one or more agents are detected → use them as install targets.
4. If no agents are detected → apply fallback behavior (Section 2.4).

The detection is implemented as a data-driven rule table
(`AGENT_RULES` in `crates/eden-skills-core/src/agents.rs`) to simplify
future additions. Detection order follows the table order in Section 2.1.

### 2.4 Fallback Behavior

When no known agent directory is detected:

- The CLI MUST default to `claude-code` as the sole target.
- The CLI MUST emit a warning: `No installed agents detected; defaulting to claude-code (~/.claude/skills/)`.

### 2.5 Target Directory Creation

If the agent's skill directory (e.g., `~/.claude/skills/`) does not exist
but the agent root directory (e.g., `~/.claude/`) does exist, the CLI
MUST create the skill directory during install.

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
| **AGT-001** | Builder | **P0** | `install` without `--target` MUST auto-detect installed agents by checking detection paths from Section 2.1. | Install on a system with `~/.claude/` and `~/.windsurf/` detects both. |
| **AGT-002** | Builder | **P0** | Detection MUST check each agent's documented detection path (Section 2.1) using `is_dir()`. | Each detection path is checked; presence → included in targets. |
| **AGT-003** | Builder | **P0** | Explicit `--target` MUST override auto-detection entirely. | `--target cursor` installs to Cursor only, even if other agents are detected. |
| **AGT-004** | Builder | **P1** | No agents detected MUST fall back to `claude-code` with warning. | System with no agent dirs → install to `~/.claude/skills/` with warning. |
| **AGT-005** | Builder | **P1** | Detection implementation MUST be data-driven (`AGENT_RULES` table) to allow ecosystem expansion without code restructuring. | Adding a new agent requires only a new row in `AGENT_RULES` and `default_agent_path`. |
| **AGT-006** | Builder | **P1** | Agents sharing `~/.agents/skills/` (Section 2.2) MUST be installable via explicit `--target` but MUST NOT generate independent detection entries. | `--target codex` works; `~/.agents/` presence detects `cursor` only (not all shared agents). |

## 6. Backward Compatibility

The detection path convention (`~/.agents/` for Cursor, Codex, and other
universal-target agents) follows the vercel-labs/skills ecosystem standard.
Systems previously configured via `vercel-labs/skills.sh` will have these
directories already present, and detection will correctly include all
corresponding agents.
