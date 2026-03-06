# SPEC_DOCKER_MANAGED.md

Docker cross-container management domain manifest.

**Related contracts:**

- `spec/phase2.95/SPEC_DOCKER_BIND.md` (bind-mount detection, mount-hint)
- `spec/phase2/SPEC_ADAPTER.md` (DockerAdapter)

## 1. Problem Statement

When host A manages container B via `--target docker:B`, and container
B also has eden-skills installed, independent operations on B can cause:

1. **Accidental file deletion** — B's `remove` deletes files from a
   read-write bind-mounted agent directory, which propagates to host A.
2. **State drift** — A's config/lock believe a skill exists in B, but
   B has removed or replaced it.
3. **Silent conflict** — B installs a different version of a skill
   that A manages, with no indication to either side.

## 2. Management Domain Manifest

### 2.1 File Location

Each agent skills directory that is managed by eden-skills MAY contain
a `.eden-managed` JSON manifest file:

```text
~/.claude/skills/
├── web-design-guidelines → ...
├── frontend-design → ...
└── .eden-managed
```

### 2.2 Format

```json
{
  "version": 1,
  "skills": {
    "web-design-guidelines": {
      "source": "external",
      "origin": "host:eden-desktop",
      "installed_at": "2026-03-07T10:30:00Z"
    },
    "frontend-design": {
      "source": "local",
      "origin": "container:agent-dev",
      "installed_at": "2026-03-07T11:00:00Z"
    }
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `version` | integer | Manifest format version (always `1`) |
| `skills` | object | Map of skill ID to ownership record |
| `skills.<id>.source` | `"external"` / `"local"` | Who manages this skill |
| `skills.<id>.origin` | string | Human-readable manager identity |
| `skills.<id>.installed_at` | ISO 8601 | When the skill was installed |

### 2.3 Origin Format

- **Host installs to Docker target:** `"host:<hostname>"`
  (from `gethostname()` or `HOSTNAME` env var)
- **Local installs:** `"local"` source with
  `"container:<container-name>"` or `"local"` as origin.

## 3. Write Lifecycle

### 3.1 When Host Installs to Docker Target

After successfully installing skill files to an agent directory
(via bind-mount host-side write or `docker cp`), the manifest
MUST be updated:

```text
read .eden-managed from agent_dir (or create empty)
set skills[skill_id] = { source: "external", origin: "host:<hostname>", installed_at: now }
write .eden-managed back
```

For bind-mount targets, the manifest is written to the host-side
path. For docker-cp targets, the manifest is written via
`docker cp`.

### 3.2 When Local Instance Installs

After a local `install` completes, the manifest MUST be updated:

```text
read .eden-managed from agent_dir (or create empty)
set skills[skill_id] = { source: "local", origin: "<identity>", installed_at: now }
write .eden-managed back
```

## 4. Guard Behavior

### 4.1 Remove Guard

When `remove` targets a skill that has `source: "external"` in the
manifest:

**Interactive mode (default):**

```text
  ⚠ Skill 'web-design-guidelines' is managed by external host (eden-desktop).

  [1] Remove from local config only (files remain)
  [2] Cancel

  Use --force to also remove files.
```

Default action is `[1]` — remove from local config, leave files and
manifest entry intact.

**`--force` flag:**

Remove files, remove manifest entry, remove from config.

**`-y` / non-interactive:**

Same as `[1]` — config-only removal. `--force` is required to
delete files.

### 4.2 Install Guard

When `install` targets a skill that has `source: "external"` in the
manifest and files already exist:

**Interactive mode (default):**

```text
  ⚠ Skill 'web-design-guidelines' already exists, managed by external host (eden-desktop).

  [1] Adopt into local config (keep existing files)
  [2] Cancel

  Use --force to overwrite files and take over management.
```

Default action is `[1]` — add to local config, update manifest to
`source: "local"`, keep existing files.

**`--force` flag:**

Re-install files, update manifest to `source: "local"`.

### 4.3 Apply / Repair Guard

When `apply` or `repair` encounters a skill whose manifest entry was
changed to `source: "local"` by the container:

**Default:** Skip the skill and emit a warning:

```text
  ⚠ Skill 'web-design-guidelines' was taken over by local management in container.
    ~> Run 'eden-skills apply --force' to reclaim, or 'eden-skills remove web-design-guidelines' to accept.
```

**`--force`:** Re-install and reset manifest to `source: "external"`.

## 5. Doctor Integration

### 5.1 Finding: `DOCKER_OWNERSHIP_CHANGED`

When a skill's manifest entry shows `source: "local"` but the host's
config still references it as a docker target:

```text
  ⚠ warning  DOCKER_OWNERSHIP_CHANGED
    Skill 'web-design-guidelines' in container 'agent-dev' was taken over by local management.
    ~> Run 'eden-skills apply --force' to reclaim, or 'eden-skills remove web-design-guidelines' to accept.
```

Severity: `warning`

### 5.2 Finding: `DOCKER_EXTERNALLY_REMOVED`

When a skill is in the host's config but missing from both the agent
directory and the manifest in the container:

```text
  ⚠ warning  DOCKER_EXTERNALLY_REMOVED
    Skill 'web-design-guidelines' was removed from container 'agent-dev'.
    ~> Run 'eden-skills apply' to re-install, or 'eden-skills remove web-design-guidelines' to accept.
```

Severity: `warning`

## 6. Manifest I/O

### 6.1 Read/Write via Bind Mount

When the agent directory has a bind mount, read and write the manifest
directly via host filesystem I/O.

### 6.2 Read/Write via Docker Exec

When no bind mount exists, use `docker exec` to read:

```bash
docker exec <container> cat <agent_dir>/.eden-managed
```

And `docker cp` to write:

```bash
# Write to temp file, then docker cp
docker cp /tmp/.eden-managed <container>:<agent_dir>/.eden-managed
```

### 6.3 Missing Manifest

If `.eden-managed` does not exist, treat all skills in the directory
as having no ownership record (no guard is triggered).

### 6.4 Corrupted Manifest

If `.eden-managed` exists but cannot be parsed, emit a warning and
proceed as if the manifest is empty.

## 7. Backward Compatibility

| Existing Feature | Phase 2.97 Behavior |
| :--- | :--- |
| Install to docker target | Unchanged + manifest write |
| Remove from docker target | Unchanged unless externally managed |
| Doctor docker findings | Additive — new finding codes |
| Non-docker installs | `.eden-managed` written to local agent dirs (source: "local") |
| `--json` output | Unchanged for existing commands |

## 8. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **DMG-001** | Builder | **P2** | Install MUST write `.eden-managed` entry after installing to an agent directory. | Manifest file exists after install. |
| **DMG-002** | Builder | **P2** | Docker install MUST set `source: "external"` in the manifest. | Manifest entry shows `"external"`. |
| **DMG-003** | Builder | **P2** | Local install MUST set `source: "local"` in the manifest. | Manifest entry shows `"local"`. |
| **DMG-004** | Builder | **P2** | `remove` MUST guard against removing externally-managed skills. | Remove of external skill shows warning and defaults to config-only. |
| **DMG-005** | Builder | **P2** | `remove --force` MUST override the guard and delete files. | Force remove deletes files and manifest entry. |
| **DMG-006** | Builder | **P2** | `install` MUST guard against overwriting externally-managed skills. | Install of existing external skill shows warning. |
| **DMG-007** | Builder | **P2** | `doctor` MUST report `DOCKER_OWNERSHIP_CHANGED` and `DOCKER_EXTERNALLY_REMOVED`. | Doctor output includes ownership findings. |
| **DMG-008** | Builder | **P2** | Missing or corrupted manifest MUST NOT block operations. | Operations proceed normally with warning. |
