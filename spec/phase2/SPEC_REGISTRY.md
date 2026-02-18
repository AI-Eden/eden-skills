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
- Version constraint matching.

## 3. Non-Goals

- Registry submission workflow or governance policy (deferred to Phase 3+).
- Web-based search interface.
- Automatic registry discovery (registries must be explicitly configured).
- Package signing or integrity verification beyond commit SHAs.
- Dependency resolution between skills.

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **ARC-201** | Builder | **P0** | Configuration MUST support multiple registry sources with priority weights. | `skills.toml` accepts `[registries]` table with `url` and `priority` per entry. |
| **ARC-202** | Shared | **P0** | Resolution logic MUST follow priority-based fallback order (highest priority first). | Installing a skill present in both registries prefers the higher-priority one. |
| **ARC-203** | Builder | **P1** | Registry indexes MUST be local Git repositories synchronized via `eden-skills update`. | `<storage.root>/registries/` contains cloned index repos after `eden-skills update`. |
| **ARC-204** | Shared | **P1** | Each registry index MUST contain a root `manifest.toml` with a `format_version` field for forward compatibility. | `registry-repo/manifest.toml` exists with `format_version = 1`. |
| **ARC-205** | Builder | **P1** | Registry sync SHOULD use shallow clone (`--depth 1`) for efficiency. | `git clone --depth 1` used for initial clone; `git fetch --depth 1` for updates. |
| **ARC-206** | Builder | **P1** | Registry resolution MUST work offline using the locally cached index when available. | With network disabled, `install <name>` resolves from cached index without errors. |
| **ARC-207** | Builder | **P0** | Version constraint matching MUST use the `semver` crate for SemVer compliance. | `^1.2`, `~1.2.3`, `>=1.0,<2.0`, `*`, and exact versions all resolve correctly. |

## 5. Architecture Decisions

### ADR-007: Index Bucketing Strategy

- **Context:** Registry indexes are file trees in a Git repo. Need a directory
  structure that scales and is easy to look up.
- **Options:**
    1. **First-character bucketing:**
       `index/g/google-search.toml`. Bucket by first character of skill name.
       - Pros: Simple; familiar (similar to early crates.io); easy manual
         browsing; O(1) path construction.
       - Cons: Uneven distribution (many skills starting with common letters
         like 's', 'c', 'a'); single-char directories accumulate many files
         at scale.
    2. **Two-character prefix bucketing:**
       `index/go/google-search.toml`. Bucket by first two characters.
       - Pros: Much more even distribution; scales to tens of thousands of
         entries; still simple path construction.
       - Cons: Overkill for early-stage registry (< 100 skills); slightly
         harder to manually navigate; empty directories for rare prefixes.
    3. **Flat directory:**
       `index/google-search.toml`. No bucketing.
       - Pros: Simplest possible; trivial to grep.
       - Cons: Filesystem performance degrades at ~1000+ files; git
         performance suffers with large flat directories.
- **Decision:** **Option 1 (First-character bucketing) with defined migration
  path to Option 2**.
- **Rationale:** The registry will start small (< 100 skills). First-character
  bucketing is simple and sufficient. The `format_version` field (ARC-204)
  enables migration to two-character bucketing when the registry grows,
  without breaking existing clients.
- **Rollback Trigger:** When any single bucket exceeds 200 entries, bump
  `format_version` to `2` and migrate to two-character bucketing.

### ADR-008: Registry Sync Strategy

- **Context:** Registry indexes must be synchronized from remote Git repos.
  The sync strategy affects first-run performance, disk usage, and offline
  behavior.
- **Options:**
    1. **Full clone + pull:**
       Standard `git clone` on first run, `git pull` on subsequent runs.
       - Pros: Simple; full history available; standard git workflow.
       - Cons: Initial clone may be slow for registries with long history;
         stores full history which is unnecessary for an index.
    2. **Shallow clone (`--depth 1`) + shallow fetch:**
       `git clone --depth 1` initially, `git fetch --depth 1` + reset for
       updates.
       - Pros: Dramatically faster clone (seconds vs minutes for large repos);
         minimal disk usage (index TOML files only, no history); only latest
         snapshot is needed for resolution.
       - Cons: No history; some git operations (`blame`, `log`) unavailable
         on local copy. These operations are not needed for index lookup.
    3. **HTTP raw fetch (GitHub API):**
       Fetch index files directly via GitHub raw content API without git.
       - Pros: No git dependency for index sync; can fetch individual files.
       - Cons: Tied to GitHub; rate limits; no atomic snapshot guarantee;
         breaks for non-GitHub registries; complex incremental sync; no
         offline-first support.
- **Decision:** **Option 2 (Shallow clone)**.
- **Rationale:** Registry indexes are append-mostly reference data. History
  is irrelevant for resolution. Shallow clone provides the fastest sync
  with minimum disk footprint. Combined with `--depth 1`, even a registry
  with thousands of TOML files syncs in seconds.
- **Trade-off:** Cannot inspect index history locally. Acceptable since
  the registry repo's full history is available on the remote.
- **Rollback Trigger:** If shallow clone/fetch proves unreliable with
  certain Git hosting providers, or if index history is needed for audit
  purposes, fall back to full clone.

### ADR-009: Version Resolution Library

- **Context:** Mode B skills specify SemVer constraints (e.g., `^2.0`, `~1.3`).
  Need a library to match constraints against available versions listed in
  the registry index.
- **Options:**
    1. **`semver` crate (by David Tolnay):**
       The de-facto Rust SemVer library.
       - Pros: Well-maintained; correct; handles all SemVer edge cases
         (pre-release ordering, build metadata); widely used (serde, cargo).
       - Cons: Additional dependency (lightweight, ~15 KB).
    2. **Custom implementation:**
       Hand-roll SemVer parsing and matching.
       - Pros: Zero dependencies.
       - Cons: Error-prone; SemVer has non-obvious edge cases (pre-release
         ordering, build metadata comparison); maintenance burden;
         likely to have bugs.
    3. **Simplified matching (exact + prefix only):**
       Support only exact versions and simple prefix matching (`1.*`).
       - Pros: Very simple to implement; no dependencies.
       - Cons: Users expect standard SemVer constraints (`^`, `~`, range);
         incompatible with npm/cargo conventions; bad UX; will need
         replacement later.
- **Decision:** **Option 1 (`semver` crate)**.
- **Rationale:** SemVer is complex and getting it wrong causes user-facing
  bugs. The `semver` crate is tiny, correct, and well-maintained. It is
  already a transitive dependency of `cargo` and most Rust tooling. The
  cost of adding it is negligible compared to the cost of SemVer bugs.
- **Trade-off:** Additional dependency (~15 KB compiled). Negligible
  compared to the cost of hand-rolling SemVer parsing and matching.
- **Rollback Trigger:** If the `semver` crate is abandoned or introduces
  breaking changes, a custom implementation limited to exact + caret +
  tilde matching is a viable fallback.

## 6. Data Model

### 6.1 Registry Index Structure

Each registry is a Git repository with a strictly structured file tree:

```text
registry-repo/
├── manifest.toml          <-- Registry metadata (format_version, name)
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

### 6.2 Registry Manifest Format

```toml
# manifest.toml (required at registry repo root)
format_version = 1
name = "eden-official"
description = "Official curated skill registry for eden-skills"
```

Required fields:

| Field | Type | Description |
| :--- | :--- | :--- |
| `format_version` | integer | Index format version. MUST be `1` for Phase 2. |
| `name` | string | Human-readable registry name. |

Optional fields:

| Field | Type | Description |
| :--- | :--- | :--- |
| `description` | string | Registry description. |
| `maintained_by` | string | Maintainer identity. |

### 6.3 Index Entry Format

Each `<skill-name>.toml` in the index contains:

```toml
[skill]
name = "google-search"
description = "Google search integration for AI agents"
repo = "https://github.com/example/google-search-skill.git"
subpath = "."
license = "MIT"

[[versions]]
version = "2.1.0"
ref = "v2.1.0"
commit = "abc123def456789012345678901234567890abcd"
yanked = false

[[versions]]
version = "2.0.0"
ref = "v2.0.0"
commit = "789abcdef012345678901234567890abcdef0123"
yanked = false
```

#### `[skill]` fields

| Field | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `name` | string | MUST | Skill name (must match filename without `.toml`). |
| `description` | string | SHOULD | Human-readable description. |
| `repo` | string | MUST | Git URL of the skill source repository. |
| `subpath` | string | MAY | Subdirectory within repo (default: `.`). |
| `license` | string | SHOULD | SPDX license identifier. |

#### `[[versions]]` fields

| Field | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `version` | string | MUST | SemVer version string. |
| `ref` | string | MUST | Git ref (tag or branch) for this version. |
| `commit` | string | MUST | Full commit SHA for integrity verification. |
| `yanked` | boolean | MAY | If `true`, version is excluded from resolution (default: `false`). |

**Design Note:** The `[[versions]]` array format (vs `[versions]` table keyed
by version string) was chosen because: (a) it preserves insertion ordering,
(b) it supports additional per-version fields naturally (`yanked`, future
`min_eden_version`), and (c) it is consistent with the `[[skills]]` pattern
in `skills.toml`. See FC-REG5 for Stage B confirmation.

### 6.4 Resolution Logic

1. `eden-skills update`:
   - Read `[registries]` from config.
   - For each registry, shallow-clone (first run) or shallow-fetch (subsequent) into
     `<storage.root>/registries/<registry-name>/`.
   - Execute registry syncs concurrently via Reactor (bounded by ARC-002).
   - Report per-registry status.

2. `eden-skills install <skill-name> [--version <constraint>]`:
   - Sort configured registries by `priority` (descending).
   - If `--registry <name>` is specified, search only that registry.
   - For each registry (in priority order):
     - Check `index/<first-char>/<skill-name>.toml`.
     - If found, parse the index entry.
   - On first match:
     - Filter `[[versions]]` to exclude `yanked = true` entries.
     - Match `--version` constraint against available versions using `semver` crate.
     - If no constraint specified, select the highest non-yanked version.
     - If constraint matches, use the matched version's `repo` + `ref` + `commit`.
     - Pass to Downloader for clone/checkout.
   - If no registry contains the skill: fail with `SKILL_NOT_FOUND` (exit 1).
   - If skill found but no version matches: fail with `VERSION_NOT_FOUND` (exit 1),
     listing available non-yanked versions.

3. `eden-skills install <skill-name>` (when already in `skills.toml`):
   - Detect existing Mode B entry for the same `name`.
   - Update version constraint if `--version` is provided.
   - Re-resolve and sync.

## 7. Failure Semantics

- **Registry Sync Failure:** `eden-skills update` MUST report per-registry sync status.
  Partial success is allowed (some registries updated, others failed).
- **Skill Not Found:** Exit code `1` with message listing all searched registries
  and their priority order.
- **Version Not Found:** Exit code `1` with message listing available non-yanked
  versions for the requested skill.
- **Ambiguous Version:** If requested version constraint matches multiple entries,
  select the highest matching version (standard SemVer resolution).
- **Corrupted Index:** If a registry index TOML fails to parse, skip that entry
  and log a warning. Do not fail the entire resolution.
- **Stale Index:** If `update` has never been run and no local cache exists,
  `install` MUST fail with an actionable error: "Registry index not found.
  Run `eden-skills update` first."
- **Manifest Missing:** If `manifest.toml` is absent in a synced registry,
  log a warning and assume `format_version = 1`.

## 8. Acceptance Criteria

1. `eden-skills update` clones/pulls configured registry repos into `<storage.root>/registries/`.
2. `eden-skills install browser-use` resolves the skill from the official registry index.
3. When a skill exists in both `official` and `forge`, the higher-priority
   registry wins.
4. The `eden-official` registry repo exists (even if initially empty) and the
   CLI can read from it.
5. Version constraint `^2.0` correctly matches `2.0.0`, `2.1.0`, but not `3.0.0` or `1.9.0`.
6. Offline resolution works when registry is cached and network is unavailable.
7. Yanked versions are excluded from resolution unless explicitly pinned.

## 9. Resolved Design Decisions (Stage B)

| ID | Item | Decision | Rationale |
| :--- | :--- | :--- | :--- |
| **FC-REG1** | Index entry required fields | **Current draft fields** (Section 6.3) confirmed. Required: `name`, `repo`, `[[versions]]` with `version`+`ref`+`commit`. SHOULD: `description`, `license`. MAY: `subpath`, `yanked`. NOT included: `keywords`, `min_eden_version` (deferred to Phase 3). | Covers the minimum needed for resolution and integrity verification. Extended fields add schema complexity without Phase 2 consumers. |
| **FC-REG2** | SemVer pre-release policy | **Pre-release versions allowed** in the index but **excluded from default constraint resolution**. Explicit pre-release pins (e.g., exact `2.1.0-beta.1`) MUST work. | Pre-release versions are valid SemVer. The `semver` crate already implements correct pre-release matching semantics (e.g., `^2.0` does not match `2.1.0-beta.1`). Allowing indexing while excluding from default resolution matches cargo behavior. |
| **FC-REG3** | Registry cache staleness threshold | **Time-based warning (7 days)**. `doctor` emits `REGISTRY_STALE` when index was last synced > 7 days ago. `install` does NOT auto-update; fails with "Run `eden-skills update` first" if no local cache exists. | Predictability over convenience. Auto-update before every resolve makes `install` non-deterministic and slow. The 7-day threshold is informative without being intrusive. |
| **FC-REG4** | Yanked version handling | **Skip silently for constraint resolution**. **Error if exact pinned version is yanked**, listing available non-yanked versions. | When resolving `^2.0`, yanked versions should be invisible. When a user explicitly pins a yanked version, they need an error. Matches cargo/npm behavior. |
| **FC-REG5** | Index version format | **`[[versions]]` array** (confirmed). | Preserves insertion ordering, supports per-version fields naturally (`yanked`, future `min_eden_version`), consistent with `[[skills]]` pattern in `skills.toml`. |
| **FC-REG6** | Registry storage path | **`<storage.root>/registries/`** (under existing config). Default resolves to `~/.local/share/eden-skills/registries/`. | Consolidates all eden-skills data under `storage.root`. If the user moves `storage.root`, registries move with it. Respects the XDG-compliant default established in Phase 1. |
