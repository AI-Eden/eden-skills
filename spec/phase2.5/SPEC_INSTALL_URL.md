# SPEC_INSTALL_URL.md

Install from URL: the core MVP feature for `eden-skills`.

**Related Phase 2 contract:** `spec/phase2/SPEC_COMMANDS_EXT.md` Section 2.2
**Rule:** This spec extends the Phase 2 `install` command with URL-mode
capabilities. The existing registry-mode (Mode B) install behavior is
unchanged.

## 1. Purpose

Allow users to install agent skills from any Git source with a single command,
matching or exceeding the ergonomics of `npx skills add <source>` from the
[vercel-labs/skills](https://github.com/vercel-labs/skills) CLI while
leveraging the deterministic reconciliation engine built in Phase 1 and 2.

## 2. Command Shape

```text
eden-skills install <source> [--skill <name>...] [--all] [--list]
                              [--id <id>] [--ref <ref>] [--target <spec>...]
                              [--config <path>] [--strict] [--json]
```

The `<source>` positional argument accepts multiple input formats (Section 3).
The command determines its execution mode (URL vs. registry) based on source
format detection (Section 3.2).

## 3. Source Format Specification

### 3.1 Supported Formats

| Format | Example | Description |
| :--- | :--- | :--- |
| GitHub shorthand | `vercel-labs/agent-skills` | Expanded to `https://github.com/{owner}/{repo}.git` |
| Full GitHub URL | `https://github.com/vercel-labs/agent-skills` | Used as-is; `.git` suffix appended if missing |
| GitHub URL with tree path | `https://github.com/vercel-labs/agent-skills/tree/main/skills/web-design` | Repo URL, ref, and subpath extracted |
| GitLab URL | `https://gitlab.com/org/repo` | Same handling as GitHub URL |
| Git SSH URL | `git@github.com:vercel-labs/agent-skills.git` | Used as-is |
| Any HTTPS Git URL | `https://example.com/repo.git` | Used as-is |
| Local path | `./my-skills`, `/opt/skills`, `~/skills` | Treated as local directory source |
| Registry name | `browser-tool` | Delegates to existing Mode B registry resolution |

### 3.2 Detection Precedence

The CLI MUST classify the `<source>` argument using the following rules,
evaluated in order:

1. **Local path:** starts with `./`, `../`, `/`, or `~` → local source mode.
2. **GitHub tree URL:** matches `https://github.com/{owner}/{repo}/tree/{ref}/{path...}`
   → extract repo URL (`https://github.com/{owner}/{repo}.git`), ref, and
   subpath from the URL components.
3. **Full URL:** contains `://` → Git URL mode.
4. **SSH URL:** matches `git@{host}:{owner}/{repo}.git` → Git URL mode.
5. **GitHub shorthand:** matches `{owner}/{repo}` (exactly one `/`, no
   protocol prefix, no whitespace) → expand to
   `https://github.com/{owner}/{repo}.git`.
6. **Registry name:** none of the above matched → Mode B registry resolution
   (existing behavior from Phase 2).

### 3.3 Skill ID Derivation

When `--id` is not provided, the skill ID MUST be auto-derived:

- From the last path segment of the repo URL, with `.git` suffix removed.
  - Example: `https://github.com/user/my-skill.git` → `my-skill`
  - Example: `vercel-labs/agent-skills` → `agent-skills`
- When `--skill` selects a single skill from a multi-skill repo, the selected
  skill name MUST be used as the ID.
- When `--all` installs multiple skills, each skill's discovered name
  (from `SKILL.md` frontmatter) MUST be used as its ID.
- ID collision with existing entries: if the derived ID already exists in
  `skills.toml`, the CLI MUST update the existing entry (upsert semantics).

### 3.4 GitHub Tree URL Parsing

When a GitHub tree URL is detected, the CLI MUST extract:

- **Repo URL:** `https://github.com/{owner}/{repo}.git`
- **Ref:** the path component immediately after `tree/`
- **Subpath:** remaining path components after ref

Example: `https://github.com/vercel-labs/agent-skills/tree/main/skills/web-design`
→ repo=`https://github.com/vercel-labs/agent-skills.git`, ref=`main`,
subpath=`skills/web-design`

## 4. Skill Discovery

### 4.1 SKILL.md Convention

A skill is defined by a directory containing a `SKILL.md` file with YAML
frontmatter containing at least `name` and `description` fields:

```markdown
---
name: my-skill
description: What this skill does
---

# My Skill

Instructions for the agent...
```

### 4.2 Search Directories

When `--skill` is not specified and no subpath is provided, the CLI MUST
search for `SKILL.md` files in the following locations within the cloned
repository, in order:

1. Root directory (if it contains `SKILL.md`)
2. `skills/` and its immediate subdirectories
3. `packages/` and its immediate subdirectories

The search MUST NOT recurse beyond two directory levels from the repository
root to bound execution time.

### 4.3 Discovery Results

| Found | Behavior |
| :--- | :--- |
| 0 skills | Install the entire repo root (or subpath if provided) as a single unnamed skill. Emit a warning: "No SKILL.md found; installing directory as-is." |
| 1 skill | Install directly without confirmation prompt. |
| 2+ skills | Enter multi-skill resolution (Section 5). |

### 4.4 Subpath Override

When `--subpath` or a GitHub tree URL provides an explicit path, discovery
is scoped to that subtree only. If a `SKILL.md` exists at the subpath root,
it is treated as a single-skill install.

## 5. Multi-Skill Resolution

When a repository contains multiple discovered skills, the CLI MUST resolve
which skills to install based on the following flag precedence:

### 5.1 `--all` Flag

Install all discovered skills without confirmation.

```bash
eden-skills install vercel-labs/agent-skills --all
```

### 5.2 `--skill` Flag

Install only the named skills. Multiple names may be provided.

```bash
eden-skills install vercel-labs/agent-skills --skill browser-tool --skill filesystem-tool
```

When a `--skill` name does not match any discovered skill, the CLI MUST
fail with an error listing available skill names.

### 5.3 Interactive Mode (No Flag)

When neither `--all` nor `--skill` is provided and stdout is a TTY,
the CLI MUST display a discovery summary and prompt:

```text
  Found    6 skills in repository:

    1. browser-tool        — Browser automation and web scraping
    2. filesystem-tool     — File system operations
    3. github-tool         — GitHub API integration
    4. search-tool         — Web search capabilities
    5. frontend-design     — Frontend design guidelines
    6. skill-creator       — Create new skills

  Install all 6 skills? [Y/n]
```

- User enters `y`, `Y`, or presses Enter → install all skills.
- User enters any other input → proceed to skill name input:

```text
  Enter skill names to install (space-separated):
  >
```

The CLI MUST validate entered names against discovered skills and reject
unknown names with an error listing available options.

### 5.4 Truncated Display

When the repository contains more than 8 discovered skills, the CLI MUST
display only the first 8 and indicate the remainder:

```text
  Found    23 skills in repository (showing first 8):

    1. browser-tool        — Browser automation and web scraping
    ...
    8. code-review         — Automated code review

    ... and 15 more (use --list to see all)

  Install all 23 skills? [Y/n]
```

### 5.5 Non-TTY Behavior

When stdout is NOT a TTY (piped, CI, redirected), the CLI MUST NOT prompt.
Behavior defaults to `--all` (install all discovered skills) to avoid
blocking automation pipelines.

### 5.6 `--list` Flag

When `--list` is provided, the CLI MUST display all discovered skills
(no truncation) and exit without installing.

```bash
eden-skills install vercel-labs/agent-skills --list
```

Output format:

```text
  Skills in vercel-labs/agent-skills:

    browser-tool        — Browser automation and web scraping
    filesystem-tool     — File system operations
    github-tool         — GitHub API integration
    ...
```

`--list` takes precedence over `--all` and `--skill`. When combined,
only listing is performed.

## 6. Execution Flow

### 6.1 URL-Mode Install Pipeline

1. **Parse source** → classify format (Section 3.2).
2. **Resolve config** → load existing `skills.toml` or auto-create if absent
   (see `SPEC_SCHEMA_P25.md` Section 3).
3. **Clone/update repository** → into `<storage.root>/<derived-id>/`.
   Use existing `source.rs` sync logic.
4. **Discover skills** → scan for `SKILL.md` in standard directories (Section 4.2).
5. **Resolve selection** → apply `--all`, `--skill`, or interactive mode (Section 5).
6. **Detect targets** → auto-detect installed agents or use `--target`
   (see `SPEC_AGENT_DETECT.md`).
7. **Persist config** → upsert selected skill entries into `skills.toml`
   as Mode A entries.
8. **Execute install** → run the standard Phase 1 source sync → plan → apply
   pipeline for the selected skills only.
9. **Report results** → display per-skill install status.

### 6.2 Config Auto-Creation

When the config file at the resolved `--config` path does not exist:

- The CLI MUST create a minimal valid config (see `SPEC_SCHEMA_P25.md`).
- The CLI MUST emit an informational message: `Created config at <path>`.
- Install proceeds using the newly created config.

### 6.3 Local Source Handling

When `<source>` is a local path:

- The CLI MUST NOT clone. Instead, it MUST use the local path as the source
  directly, consistent with `file://` URL semantics in Phase 1.
- The CLI MUST still persist a Mode A entry to `skills.toml` with the
  resolved absolute path as `source.repo`.
- Skill discovery (Section 4) applies to the local directory.

## 7. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **MVP-001** | Builder | **P0** | `install` MUST accept GitHub shorthand (`owner/repo`) and expand to HTTPS URL. | `eden-skills install vercel-labs/agent-skills` clones from GitHub. |
| **MVP-002** | Builder | **P0** | `install` MUST accept full GitHub/GitLab HTTPS URLs. | `eden-skills install https://github.com/user/repo` succeeds. |
| **MVP-003** | Builder | **P0** | `install` MUST accept GitHub tree URLs and extract repo, ref, and subpath. | Tree URL installs skill from correct subpath. |
| **MVP-004** | Builder | **P1** | `install` MUST accept Git SSH URLs. | SSH URL source is cloned and installed. |
| **MVP-005** | Builder | **P1** | `install` MUST accept local paths as source. | `eden-skills install ./my-skills` installs from local directory. |
| **MVP-006** | Builder | **P0** | Source format detection MUST follow Section 3.2 precedence. | Each format is correctly classified without ambiguity. |
| **MVP-007** | Builder | **P0** | Skill ID MUST be auto-derived from source with `--id` override. | Derived ID matches expected pattern; `--id` overrides it. |
| **MVP-008** | Builder | **P0** | `install` MUST auto-create config file if it does not exist. | Install on fresh system creates config and completes. |
| **MVP-009** | Builder | **P0** | `install` MUST discover `SKILL.md` files in standard directories (Section 4.2). | Multi-skill repo correctly lists discovered skills. |
| **MVP-010** | Builder | **P0** | `--list` flag MUST display discovered skills without installing. | `--list` output shows skill names and descriptions; no filesystem changes. |
| **MVP-011** | Builder | **P0** | `--all` flag MUST install all discovered skills without confirmation. | `--all` installs every discovered skill. |
| **MVP-012** | Builder | **P0** | `--skill` flag MUST install only named skills. | `--skill browser-tool` installs only that skill. |
| **MVP-013** | Builder | **P0** | Interactive mode MUST show discovered skills and prompt for confirmation. | TTY prompt displays skill list; `y` installs all; `n` prompts for names. |
| **MVP-014** | Builder | **P1** | Non-TTY MUST default to `--all` behavior (no blocking prompts). | Piped command installs all without prompting. |
| **MVP-015** | Builder | **P1** | Single-skill repos MUST skip confirmation and install directly. | Repo with one SKILL.md installs without prompt. |

## 8. Backward Compatibility

| Existing Feature | Phase 2.5 Behavior |
| :--- | :--- |
| `install <skill-name>` (Mode B registry) | Unchanged. Registry mode is selected when source does not match URL patterns. |
| `install --version`, `--registry` | Unchanged. These flags apply to registry mode only. |
| `install --dry-run` | Unchanged for registry mode. In URL mode, `--dry-run` shows what would be installed without executing. |
| `install --target` | Extended. In URL mode, `--target` overrides agent auto-detection. |
| `add`, `apply`, `plan`, `doctor`, `repair` | Unchanged. |

## 9. Future Scope (Not in Phase 2.5)

- `install` from private repos with SSH key or token authentication.
- Interactive skill selection with `dialoguer` multi-select (fuzzy search).
- `install` with dependency resolution between skills.
- Plugin manifest discovery (`.claude-plugin/marketplace.json`).
