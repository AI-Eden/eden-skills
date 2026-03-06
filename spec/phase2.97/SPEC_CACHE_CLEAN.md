# SPEC_CACHE_CLEAN.md

Cache cleanup command and orphan detection.

**Related contracts:**

- `spec/phase2.95/SPEC_PERF_SYNC.md` (repo-level cache at `.repos/`)

## 1. Problem Statement

Phase 2.95 introduced a repo-level cache at `storage_root/.repos/`.
When skills are removed, the corresponding `.repos/<cache-key>/`
directory is not cleaned up. Over time, orphaned cache entries
accumulate and consume disk space.

Additionally, `install` discovery operations create temporary
directories under the system temp dir (`/tmp/eden-skills-discovery-*`)
that may be left behind if the process is interrupted.

## 2. `clean` Command

### 2.1 Invocation

```bash
eden-skills clean [--dry-run] [--json] [--config <path>]
```

### 2.2 Behavior

1. **Load config** — read `skills.toml` and compute the set of
   referenced repo cache keys:

   ```text
   referenced = { repo_cache_key(skill.source.repo, skill.source.ref)
                   for skill in config.skills
                   if !is_local_source(skill) }
   ```

2. **Scan cache directory** — list all entries in `storage_root/.repos/`.

3. **Identify orphans** — entries present on disk but not in the
   referenced set.

4. **Scan discovery temp dirs** — list all `eden-skills-discovery-*`
   directories under the system temp dir.

5. **Delete orphans** (unless `--dry-run`):
   - Remove each orphaned `.repos/<key>/` directory recursively.
   - Remove each stale discovery temp directory.

6. **Output:**
   - Human mode: report count of removed entries and freed space.
   - JSON mode: array of removed paths.
   - Dry-run mode: list what would be removed without deleting.

### 2.3 Human Mode Output

```text
  Clean  3 orphaned cache entries removed
         1 stale discovery directory removed

  ✓ Freed 142 MB
```

With `--dry-run`:

```text
  Clean  would remove 3 orphaned cache entries:
           ~/.eden-skills/skills/.repos/github.com_example_repo@main
           ~/.eden-skills/skills/.repos/github.com_old_repo@v1
           ~/.eden-skills/skills/.repos/github.com_stale_repo@dev

         would remove 1 stale discovery directory:
           /tmp/eden-skills-discovery-12345-...

  ✓ Dry run complete — no files deleted
```

### 2.4 JSON Mode Output

```json
{
  "action": "clean",
  "dry_run": false,
  "removed_cache_entries": [
    "~/.eden-skills/skills/.repos/github.com_example_repo@main"
  ],
  "removed_discovery_dirs": [
    "/tmp/eden-skills-discovery-12345-..."
  ],
  "freed_bytes": 148897792
}
```

## 3. `remove --auto-clean`

### 3.1 Flag

`eden-skills remove [skills...] --auto-clean`

### 3.2 Behavior

After the normal remove operation completes, run the `clean` logic
(Section 2.2) automatically. Output the clean summary after the
remove summary.

### 3.3 Interaction with `--json`

When `--json` is active, the `remove` JSON payload gains an additional
`"clean"` field:

```json
{
  "action": "remove",
  "removed": ["skill-a"],
  "clean": {
    "removed_cache_entries": [...],
    "freed_bytes": 0
  }
}
```

## 4. `doctor` Orphan Cache Finding

### 4.1 Finding Code

`ORPHAN_CACHE_ENTRY`

### 4.2 Severity

`info`

### 4.3 Behavior

`doctor` MUST scan `storage_root/.repos/` and report orphaned entries
that are not referenced by any configured skill.

```text
  ℹ info  ORPHAN_CACHE_ENTRY
    Orphaned cache entry: .repos/github.com_old_repo@main
    ~> Run 'eden-skills clean' to free disk space.
```

### 4.4 JSON Doctor Output

```json
{
  "code": "ORPHAN_CACHE_ENTRY",
  "severity": "info",
  "skill_id": "",
  "target_path": ".repos/github.com_old_repo@main",
  "message": "Orphaned cache entry not referenced by any configured skill",
  "remediation": "Run 'eden-skills clean' to free disk space."
}
```

## 5. Backward Compatibility

| Existing Feature | Phase 2.97 Behavior |
| :--- | :--- |
| `remove` without `--auto-clean` | Unchanged — no cache cleanup |
| `doctor` existing findings | Unchanged — new finding is additive |
| `.repos/` directory structure | Unchanged |
| `--json` for `remove` | Backward-compatible (new optional field) |

## 6. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **CCL-001** | Builder | **P1** | `clean` MUST identify orphaned `.repos/` entries not referenced by config. | Orphaned entries are listed and removed. |
| **CCL-002** | Builder | **P1** | `clean` MUST remove stale `eden-skills-discovery-*` temp directories. | Discovery temp dirs are cleaned up. |
| **CCL-003** | Builder | **P1** | `clean --dry-run` MUST list removals without deleting. | Dry run lists paths; no files are deleted. |
| **CCL-004** | Builder | **P1** | `clean --json` MUST output machine-readable removal report. | JSON output matches Section 2.4 schema. |
| **CCL-005** | Builder | **P1** | `remove --auto-clean` MUST run clean logic after removal. | Remove with `--auto-clean` cleans orphaned cache. |
| **CCL-006** | Builder | **P1** | `doctor` MUST report `ORPHAN_CACHE_ENTRY` for orphaned cache dirs. | Doctor output includes orphan finding. |
| **CCL-007** | Builder | **P2** | `clean` MUST report freed disk space in human mode. | Human output shows freed bytes. |
