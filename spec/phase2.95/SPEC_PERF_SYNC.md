# SPEC_PERF_SYNC.md

Source sync performance optimization: repo-level cache, discovery clone
reuse, batch-parallel sync, and cross-command migration.

**Related contracts:**

- `spec/phase2/SPEC_REACTOR.md` (concurrency model)
- `spec/phase2.5/SPEC_INSTALL_URL.md` (URL-mode install flow)
- `spec/phase2.7/SPEC_LOCK.md` (lock diff semantics)
- `spec/phase2.9/SPEC_UPDATE_EXT.md` (Mode A refresh)

## 1. Problem Statement

The current source sync model clones one full Git repository per
skill into `storage_root/{skill_id}/`. This creates three compounding
performance bottlenecks:

1. **Redundant clones (severe):** Installing 30 skills from
   `vercel-labs/agent-skills` executes 30 separate `git clone --depth 1`
   operations against the same remote, each downloading the full repo
   into a distinct directory.
2. **Discarded discovery clone (moderate):** The URL-mode install flow
   clones the repo into a temporary directory for SKILL.md discovery,
   then deletes it. A second clone follows during source sync.
3. **Serial per-skill sync (moderate):** The install loop calls
   `sync_sources_async()` once per skill with a single-skill config.
   The reactor receives one task per call, defeating concurrency.

These bottlenecks also affect `update`, `apply`, and `repair` via
their shared `sync_sources_async` call path.

## 2. Repo-Level Cache

### 2.1 Directory Structure

A new `.repos/` directory inside `storage_root` holds one checkout
per unique `(repo_url, ref)` pair:

```
storage_root/
  .repos/
    {cache_key}/
      .git/
      skills/
        web-design-guidelines/SKILL.md
        browser-tool/SKILL.md
```

Skills reference their source via `storage_root/.repos/{cache_key}/{subpath}`.

### 2.2 Cache Key Derivation

The cache key is derived from the normalized repo URL and sanitized
ref, joined by `@`:

```
cache_key = normalize_repo_url(url) + "@" + sanitize_ref(ref)
```

**`normalize_repo_url(url)`:**

1. Strip scheme prefix (`https://`, `http://`, `git://`, `ssh://`).
2. Strip `git@` prefix and replace the first `:` with `/`.
3. Strip trailing `.git` suffix.
4. Replace `/` with `_`.
5. Lowercase the result.

Examples:

| Input | Normalized |
| :--- | :--- |
| `https://github.com/vercel-labs/agent-skills.git` | `github.com_vercel-labs_agent-skills` |
| `git@github.com:user/repo.git` | `github.com_user_repo` |
| `https://github.com/AI-Eden/eden-skills` | `github.com_ai-eden_eden-skills` |

**`sanitize_ref(ref)`:**

1. Replace `/` with `_`.
2. Remove characters outside `[a-zA-Z0-9._-]`.

Examples: `main` → `main`, `v2.0` → `v2.0`, `refs/heads/main` → `refs_heads_main`.

**Full cache key example:**

`github.com_vercel-labs_agent-skills@main`

### 2.3 Source Path Resolution

The function `resolve_skill_source_path(storage_root, skill)` MUST
return the effective source path for a skill:

```
storage_root/.repos/{cache_key}/{skill.source.subpath}
```

This replaces the current `storage_root/{skill.id}/{skill.source.subpath}`.

`build_plan()` in `plan.rs` and `execute_install_plan()` in
`install.rs` MUST use this function for source path resolution.

### 2.4 Grouping Strategy

Before source sync, skills MUST be grouped by their `(repo_url, ref)`
pair (using the normalized cache key). Each unique group produces
exactly one `SyncTask`. The reactor runs all tasks in parallel with
the configured concurrency limit.

```rust
// Pseudocode
let groups: HashMap<String, Vec<&SkillConfig>> = group_by_cache_key(&config.skills);
let tasks: Vec<RepoSyncTask> = groups.into_iter().map(|(key, skills)| {
    RepoSyncTask { cache_key: key, repo_url: skills[0].source.repo, reference: skills[0].source.ref, ... }
}).collect();
reactor.run_phase_a(tasks, |task| sync_one_repo(task)).await;
```

## 3. Discovery Clone Reuse

### 3.1 Current Flow (Wasteful)

1. `discover_remote_skills_via_temp_clone()` → `git clone --depth 1`
   into `/tmp/eden-skills-discovery-*` → discover SKILL.md files →
   `TempDiscoveryCheckout::drop()` deletes the temp directory.
2. `sync_sources_async()` → `clone_repo()` clones the same repo again
   into `storage_root/.repos/{cache_key}/`.

### 3.2 Optimized Flow

1. `discover_remote_skills_via_temp_clone()` → `git clone --depth 1`
   into a temporary directory → discover SKILL.md files.
2. **Instead of dropping**, move (rename) the temp directory to the
   repo cache location: `storage_root/.repos/{cache_key}/`.
3. During `sync_sources_async()`, the cache directory already exists.
   `sync_one_repo()` detects `.git/` and runs `update_repo()` (fetch)
   instead of `clone_repo()`.

### 3.3 Cross-Filesystem Fallback

If `fs::rename()` fails (e.g., `/tmp` and `storage_root` are on
different filesystems), fall back to a fresh clone in the cache
location. The temp directory is deleted as before.

### 3.4 Local Source Install

Local source installs (`install_local_url_mode_async`) do NOT use the
repo cache. They continue to copy the source into
`storage_root/{skill_id}/` via `stage_local_source_into_storage()`.
This path is unchanged.

## 4. Batch Sync in Install

### 4.1 Current Serial Loop

```rust
for (index, skill_id) in selected_ids.iter().enumerate() {
    let single = select_single_skill_config(&selected_config, skill_id)?;
    let sync = sync_sources_async(&single, &config_dir).await?;
    // ...
    execute_install_plan(&single, &config_dir, strict)?;
}
```

### 4.2 Optimized Batch

```rust
// Phase A: sync all unique repos in parallel
let sync_summary = sync_sources_async(&selected_config, &config_dir).await?;

// Phase B: execute install plans sequentially (filesystem mutations)
for skill_id in &selected_ids {
    let single = select_single_skill_config(&selected_config, skill_id)?;
    execute_install_plan(&single, &config_dir, strict)?;
}
```

`sync_sources_async` receives the full selected config. Internally,
it groups by cache key (Section 2.4) and runs parallel sync tasks.

## 5. Lock Diff Skip Optimization

### 5.1 Current Behavior

`apply` and `repair` call `sync_sources_async(&config, ...)` for ALL
skills, even those whose lock status is `Unchanged`. The `update_repo`
function always executes `git fetch --all --prune`.

### 5.2 Optimized Behavior

When a lock file is available and valid, `sync_sources_async` SHOULD
accept an optional `skip_repos: HashSet<String>` parameter containing
cache keys whose lock status is `Unchanged`.

For each `SyncTask`, if its cache key is in `skip_repos`, the task
returns `SyncOutcome::Skipped` immediately without network I/O.

This optimization applies to `apply` only. `repair` always does a
full sync (its contract requires force-reinstall).

## 6. Migration and Backward Compatibility

### 6.1 Detection

On startup of any source-sync operation, the sync layer MUST check
whether the repo cache directory (`storage_root/.repos/`) exists.

### 6.2 Gradual Migration

- If `.repos/` does not exist, create it.
- Old per-skill directories (`storage_root/{skill_id}/.git`) are
  NOT automatically migrated. They become orphaned.
- On next `apply`, `install`, or `update`, new syncs populate
  `.repos/` and symlinks are updated to point to the new paths.
- Old per-skill directories MAY be cleaned up by `repair` or
  `doctor` (as orphan warnings).

### 6.3 Lock File

The lock file format is unchanged. The `installed_at` timestamp and
`resolved_commit` fields continue to record the latest sync state.
No fields reference the internal cache path.

### 6.4 skills.toml

No changes to `skills.toml` format. The `storage.root` field
continues to point to the storage root. The `.repos/` subdirectory
is an internal implementation detail.

## 7. Cross-Command Migration

### 7.1 update

`update_async()` in `update.rs` currently calls
`sync_sources_async()` for Mode A refresh. After this change, it
MUST use the repo-level cache and grouping strategy. The
`refresh_mode_a_skills()` function MUST resolve repo cache paths
instead of per-skill paths.

### 7.2 apply / repair

`apply_async()` and `repair_async()` in `reconcile.rs` call
`sync_sources_async()` with the full config. After this change,
they benefit automatically from repo-level caching and grouping.
`apply` additionally benefits from lock diff skip (Section 5).

### 7.3 plan / doctor

`build_plan()` and `verify_targets()` resolve source paths at
read time. They MUST use `resolve_skill_source_path()` (Section 2.3)
to find sources in the repo cache.

## 8. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **PSY-001** | Builder | **P0** | Source sync MUST use a repo-level cache at `storage_root/.repos/{cache_key}/`. | Cache directory created; skills sharing the same `(repo_url, ref)` produce exactly one `SyncTask`. |
| **PSY-002** | Builder | **P0** | Cache key MUST be derived from normalized URL + sanitized ref per Section 2.2. | Normalization tests cover all listed examples. |
| **PSY-003** | Builder | **P0** | URL-mode install MUST reuse the discovery clone by moving it to the cache location. | No second `git clone` after discovery when cache is empty. |
| **PSY-004** | Builder | **P0** | Install sync MUST batch all selected skills into one `sync_sources_async` call with reactor parallelism. | Reactor receives N tasks (one per unique cache key), not one task per skill. |
| **PSY-005** | Builder | **P1** | `apply` SHOULD skip source sync for repos whose lock status is `Unchanged`. | `SyncOutcome::Skipped` returned for unchanged repos; no network I/O. |
| **PSY-006** | Builder | **P0** | `update`, `apply`, and `repair` MUST resolve source paths via the repo cache. | Source path resolution uses `resolve_skill_source_path()`. |
| **PSY-007** | Builder | **P1** | Migration from per-skill directories to repo cache MUST be gradual and non-destructive. | Old directories are not deleted; new syncs populate `.repos/`. |
| **PSY-008** | Builder | **P2** | Copy-mode `copy_content_equal` SHOULD use mtime + size fast path before byte comparison. | Fast path returns `Noop` without reading file contents when mtime and size match. |

## 9. Backward Compatibility

| Existing Feature | Phase 2.95 Behavior |
| :--- | :--- |
| `skills.toml` format | Unchanged. |
| `skills.lock` format | Unchanged (field values unchanged). |
| `--json` output | Unchanged. |
| Local source installs | Unchanged (no repo cache for local paths). |
| `--copy` mode | Unchanged (copy still reads from repo cache subpath). |
| Exit codes | Unchanged. |
