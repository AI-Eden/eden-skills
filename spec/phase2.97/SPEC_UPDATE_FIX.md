# SPEC_UPDATE_FIX.md

Fix `update` Mode A refresh concurrency bug.

**Related contracts:**

- `spec/phase2.95/SPEC_PERF_SYNC.md` (repo-level cache model)
- `spec/phase2.9/SPEC_UPDATE_EXT.md` (update dual-layer refresh)

## 1. Problem Statement

Phase 2.95 introduced repo-level caching (`PSY-001`). Multiple skills
from the same repository now share a single `.repos/<cache-key>/`
directory. However, the `update` command's `build_mode_a_refresh_tasks`
creates one `SkillRefreshTask` per skill and runs them in parallel via
the reactor. When several tasks target the same `.git` directory
concurrently, Git's shallow file (`shallow`) is modified by racing
`git fetch --depth 1` invocations, producing:

- `fatal: shallow file has changed since we read it`
- `Another git process seems to be running in this repository`

The `source.rs` sync path correctly deduplicates by `repo_cache_key`.
The update path does not.

## 2. Fix: Deduplicate Refresh Tasks

### 2.1 Grouping

`build_mode_a_refresh_tasks` MUST group skills by `repo_cache_key`
(same function used in `source.rs`). For each unique key, exactly
**one** `SkillRefreshTask` is created.

```text
skills: [A, B, C, D, E]  (all from same repo@main)
  ↓ group by repo_cache_key
tasks: [{ key: "github.com_vercel-labs_agent-skills@main", skills: [A,B,C,D,E] }]
  ↓ reactor
fetches: 1 (not 5)
```

### 2.2 Result Broadcasting

After the single fetch completes, the `FETCH_HEAD` SHA is compared
against the local `HEAD` SHA. The resulting `SkillRefreshStatus` is
**broadcast** to all skills sharing that cache key.

### 2.3 Stale Lock Cleanup

Before executing `git fetch`, the refresh task SHOULD check for stale
`.git/shallow.lock` and `.git/index.lock` files. If a lock file exists
and is older than 60 seconds, it SHOULD be deleted with a warning.
This prevents cascading failures from prior interrupted operations.

## 3. Backward Compatibility

| Existing Feature | Phase 2.97 Behavior |
| :--- | :--- |
| `update` human-mode table | Unchanged — one row per skill |
| `update --json` schema | Unchanged — one entry per skill in `skills` array |
| `update --apply` lifecycle | Unchanged |
| Local-source skills | Unchanged — not grouped (no repo cache) |
| Registry-mode refresh | Unchanged (separate code path) |

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **UFX-001** | Builder | **P0** | `build_mode_a_refresh_tasks` MUST group tasks by `repo_cache_key` and emit one fetch per unique key. | Two skills from the same repo produce one git fetch. |
| **UFX-002** | Builder | **P0** | After fetch, the refresh status MUST be broadcast to all skills sharing the same cache key. | Update table shows correct per-skill status for all grouped skills. |
| **UFX-003** | Builder | **P1** | Before fetch, stale `.git/*.lock` files older than 60 seconds SHOULD be removed with a warning. | Consecutive `update` calls do not fail with "Another git process" error. |
