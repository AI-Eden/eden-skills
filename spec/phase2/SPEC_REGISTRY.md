# SPEC_REGISTRY.md

Normative specification for the Phase 2 double-track registry system.

## 1. Purpose

Define the registry resolution architecture that enables skill discovery and
installation by name (e.g., `eden-skills install google-search`) instead of requiring
explicit Git URLs. The system supports two tracks: `official` (curated) and
`forge` (community).

## 2. Scope

- Registry configuration schema (`[registries]` table in `skills.toml`).
- Registry index format (Git-based TOML index).
- Resolution logic (priority-based fallback).
- `eden-skills update` behavior for registry synchronization.

## 3. Non-Goals

- Registry submission workflow or governance policy (deferred to Phase 3+).
- Web-based search interface.
- Automatic registry discovery (registries must be explicitly configured).

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **ARC-201** | Builder | **P0** | Configuration MUST support multiple registry sources with priority weights. | `skills.toml` accepts `[registries]` table with `url` and `priority` per entry. |
| **ARC-202** | Shared | **P0** | Resolution logic MUST follow `Official -> Forge -> Failure` fallback order by default (determined by priority weight). | Installing a skill present in both registries prefers the higher-priority one. |
| **ARC-203** | Builder | **P1** | Registry indexes MUST be local Git repositories synchronized via `eden-skills update`. | `~/.eden-skills/registries/` contains cloned index repos after `eden-skills update`. |

## 5. Data Model

### 5.1 Registry Index Structure

Each registry is a Git repository with a strictly structured file tree:

```text
registry-repo/
├── index/
│   ├── a/
│   │   └── agent-search.toml
│   ├── b/
│   │   └── browser-use.toml
│   └── g/
│       └── google-search.toml
├── README.md
└── POLICY.md
```

Index entries are bucketed by first character of the skill name.

### 5.2 Index Entry Format (Draft)

Each `<skill-name>.toml` in the index contains:

```toml
[skill]
name = "google-search"
description = "Google search integration for AI agents"
repo = "https://github.com/example/google-search-skill.git"
subpath = "."
ref = "v2.1.0"
license = "MIT"

[versions]
"2.1.0" = { ref = "v2.1.0", commit = "abc123..." }
"2.0.0" = { ref = "v2.0.0", commit = "def456..." }
```

### 5.3 Resolution Logic

1. `eden-skills update`: Pulls latest commits from all configured registry repos (concurrently via Reactor).
2. `eden-skills install <skill-name>`:
   - Sort registries by priority (descending).
   - For each registry, check `index/<first-char>/<skill-name>.toml`.
   - On first match, read Git URL and version info.
   - Pass to Downloader for clone/checkout.
   - If no registry contains the skill, fail with `SKILL_NOT_FOUND`.

## 6. Failure Semantics

- **Registry Sync Failure:** `eden-skills update` MUST report per-registry sync status.
  Partial success is allowed (some registries updated, others failed).
- **Skill Not Found:** Exit code `1` with message listing all searched registries.
- **Ambiguous Version:** If requested version constraint matches no entry, fail
  with available versions listed.
- **Corrupted Index:** If a registry index TOML fails to parse, skip that entry
  and log a warning. Do not fail the entire resolution.

## 7. Acceptance Criteria

1. `eden-skills update` clones/pulls configured registry repos into `~/.eden-skills/registries/`.
2. `eden-skills install browser-use` resolves the skill from the official registry index.
3. When a skill exists in both `official` and `forge`, the higher-priority
   registry wins.
4. The `eden-official` registry repo exists (even if initially empty) and the
   CLI can read from it.
