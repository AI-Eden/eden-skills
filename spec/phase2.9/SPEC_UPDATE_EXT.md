# SPEC_UPDATE_EXT.md

Extension of the `update` command to refresh Mode A skill sources.

**Related contracts:**

- `spec/phase2/SPEC_COMMANDS_EXT.md` (Phase 2 `update` definition)
- `spec/phase2/SPEC_REGISTRY.md` (registry sync behavior)

## 1. Problem Statement

The Phase 2 `update` command only synchronizes registry indexes
(`[registries]` entries in `skills.toml`). Skills installed via URL
(Mode A — the dominant first-use scenario) have no update mechanism:

- `update` outputs "no registries configured; skipping update" and
  exits immediately.
- `apply` skips source sync for skills whose lock status is
  `Unchanged`, so it does not pull upstream changes either.

Users who install skills with `eden-skills install owner/repo`
reasonably expect `update` to check for upstream changes and offer
to apply them.

## 2. Extended Semantics

### 2.1 Dual-Layer Refresh

After Phase 2.9, `update` performs two sequential refresh operations:

1. **Registry sync** (existing behavior, unchanged) — for each
   configured `[registries]` entry, shallow-clone or shallow-fetch
   the index repository.
2. **Mode A skill refresh** (new behavior) — for each skill in the
   config whose `source.repo` is NOT a `registry://` URL, fetch the
   latest commits from the remote for the configured ref.

Both layers are optional: if no registries exist, layer 1 is skipped
silently. If no Mode A skills exist, layer 2 is skipped silently.
When BOTH are empty, `update` MUST output a helpful message instead
of the current bare warning.

### 2.2 Mode A Refresh Mechanism

For each Mode A skill:

1. Resolve the cached repository directory:
   `<storage.root>/<skill.id>/`.
2. If the directory does not exist or has no `.git/`:
   - Status = `missing` (the skill has never been synced).
3. If the directory exists:
   - Read `HEAD` SHA before fetch.
   - Execute `git -C <dir> fetch --depth 1 origin <ref>`.
   - Read `FETCH_HEAD` SHA.
   - If `HEAD` == `FETCH_HEAD`: status = `up-to-date`.
   - Else: status = `new-commit`.
4. Do NOT `git reset --hard FETCH_HEAD`. The fetch-only approach
   keeps the local state unchanged until an explicit `apply` or
   `update --apply` is run.

### 2.3 `--apply` Flag

When `--apply` is passed:

1. After the refresh report, execute the full apply lifecycle
   (source sync → safety → orphan removal → plan → verify → lock)
   for all skills that reported `new-commit` or `missing`.
2. The output seamlessly transitions from the refresh summary to the
   install results (same format as `apply`).

When `--apply` is NOT passed:

1. `update` is read-only — no filesystem mutations beyond the
   `git fetch` operations and registry sync.
2. If any skills have `new-commit` status, a hint line MUST be
   appended suggesting `update --apply` or `apply`.

## 3. Output Specification

### 3.1 Full Output (Registries + Skills)

```text
  Update   2 registries synced

 ┌──────────┬─────────┬────────┐
 │ Registry │ Status  │ Detail │
 ╞══════════╪═════════╪════════╡
 │ official │ updated │        │
 │ forge    │ skipped │        │
 └──────────┴─────────┴────────┘

  Refresh  4 skills checked

 ┌──────────────────────────────┬────────────┐
 │ Skill                        │ Status     │
 ╞══════════════════════════════╪════════════╡
 │ vercel-composition-patterns  │ new commit │
 │ vercel-react-best-practices  │ up-to-date │
 │ vercel-react-native-skills   │ up-to-date │
 │ web-design-guidelines        │ new commit │
 └──────────────────────────────┴────────────┘

  ✓ 0 registry failures, 2 skills have updates [1.3s]
  → Run 'eden-skills update --apply' or 'eden-skills apply' to install.
```

### 3.2 No Registries, Skills Only

```text
  Refresh  4 skills checked

 ┌──────────────────────────────┬────────────┐
 │ Skill                        │ Status     │
 ╞══════════════════════════════╪════════════╡
 │ vercel-composition-patterns  │ up-to-date │
 │ ...                          │ ...        │
 └──────────────────────────────┴────────────┘

  ✓ All skills up to date [0.8s]
```

### 3.3 Nothing Configured

```text
  Update   no skills or registries configured

  → Run 'eden-skills install <owner/repo>' to get started.
```

### 3.4 With `--apply`

```text
  Update   2 registries synced
  Refresh  4 skills checked, 2 have updates
  Syncing  [1/2] vercel-composition-patterns ✓
           [2/2] web-design-guidelines ✓
  Install  ✓ vercel-composition-patterns
             ├─ ~/.claude/skills/vercel-composition-patterns (symlink)
             └─ ~/.codex/skills/vercel-composition-patterns (symlink)
           ✓ web-design-guidelines
             ├─ ~/.claude/skills/web-design-guidelines (symlink)
             └─ ~/.codex/skills/web-design-guidelines (symlink)
  ✓ 2 skills updated [3.1s]
```

### 3.5 Skill Refresh Status Labels (Plain Table Cells)

Skill refresh status values are rendered as plain text labels in table
cells (no ANSI styling attributes in table content).

| Status | Condition |
| :--- | :--- |
| `new commit` | Remote HEAD differs from local HEAD |
| `up-to-date` | Remote HEAD matches local HEAD |
| `missing` | Cached repo directory does not exist |
| `failed` | `git fetch` failed |

### 3.6 JSON Output

When `--json` is set, the payload extends the existing registry JSON
with an additional `skills` array:

```json
{
  "registries": [ ... ],
  "skills": [
    {
      "id": "vercel-composition-patterns",
      "status": "new-commit",
      "local_sha": "abc123",
      "remote_sha": "def456"
    }
  ],
  "failed": 0,
  "elapsed_ms": 1300
}
```

When `--apply` is combined with `--json`, the `skills` entries gain
an `applied` boolean field.

## 4. Concurrency

Mode A skill refresh tasks MUST be executed through the reactor
(`SkillReactor::run_phase_a`) with the same concurrency bounds as
registry sync. The `--concurrency` flag applies to both layers.

## 5. New CLI Flag

| Flag | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `--apply` | bool | `false` | After refresh, reconcile targets for skills with available updates |

The `--apply` flag MUST be added to `UpdateArgs` in `lib.rs` and
threaded through to `UpdateRequest`.

## 6. Interaction with Other Commands

| Command | Relationship |
| :--- | :--- |
| `apply` | `update --apply` is equivalent to `update` + `apply` for changed skills only. Full `apply` still handles lock diff, orphan removal, and safety analysis for ALL skills. |
| `repair` | Unaffected. `repair` always force-reinstalls everything. |
| `doctor` | `REGISTRY_STALE` finding now also considers skill source staleness (last-fetch timestamp). |
| `install` | Unaffected. Install always clones fresh. |

## 7. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **UPD-001** | Builder | **P0** | `update` MUST refresh Mode A skill sources via `git fetch`. | Mode A skills in config trigger fetch; status reported. |
| **UPD-002** | Builder | **P0** | `update` without `--apply` MUST NOT mutate local source state beyond fetch. | No `git reset`, no file copy, no symlink changes. |
| **UPD-003** | Builder | **P0** | `update --apply` MUST reconcile targets for skills with `new-commit` or `missing` status. | Changed skills get source sync + plan + install. |
| **UPD-004** | Builder | **P0** | Skill refresh results MUST render as a table with Skill and Status columns. | Table output present in human mode. |
| **UPD-005** | Builder | **P1** | Status values MUST render as plain labels in table cells (no ANSI styling attributes). | Skill status cells are plain text (`new commit`, `up-to-date`, `missing`, `failed`). |
| **UPD-006** | Builder | **P0** | When no registries AND no skills exist, output MUST include install guidance. | Section 3.3 message displayed. |
| **UPD-007** | Builder | **P1** | `--json` output MUST extend existing schema with `skills` array. | JSON includes skill refresh statuses. |
| **UPD-008** | Builder | **P1** | Skill refresh MUST use reactor concurrency. | Refresh tasks run through `SkillReactor`. |

## 8. Backward Compatibility

| Existing Feature | Phase 2.9 Behavior |
| :--- | :--- |
| `update` with registries configured | Registry sync unchanged. Skill refresh appended as second section. |
| `update --json` | JSON schema extended (additive), not breaking. |
| `--concurrency` flag | Applies to both registry sync and skill refresh. |
| Exit codes | Unchanged. Registry failures still exit 0 with per-registry error report. |
