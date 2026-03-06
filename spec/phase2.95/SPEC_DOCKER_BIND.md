# SPEC_DOCKER_BIND.md

Docker container agent auto-detection, bind-mount optimization,
host-side symlink, and `docker mount-hint` subcommand.

**Related contracts:**

- `spec/phase2/SPEC_ADAPTER.md` (DockerAdapter, ADR-001 docker cp)
- `spec/phase2/SPEC_COMMANDS_EXT.md` (install --target docker:)
- `spec/phase2.5/SPEC_AGENT_DETECT.md` (local agent auto-detection)

## 1. Problem Statement

The `DockerAdapter` has two fundamental gaps:

**1a. No agent auto-detection inside containers.** When a user runs
`install --target docker:my-container`, the CLI hardcodes a single
`ClaudeCode` target. It does not detect which agents are actually
installed inside the container. This is inconsistent with local
behavior, where `detect_installed_agent_targets()` scans the host
for all known agent directories.

**1b. Always uses `docker cp` (hard copy).** Even when the target
path is served by a bind mount from the host, the adapter copies
files into the container instead of creating a symlink on the host.

## 2. Container Agent Auto-Detection

### 2.1 Mechanism

`DockerAdapter` MUST expose a `detect_agents()` method that discovers
installed agents inside a running container, mirroring the local
`detect_installed_agent_targets()` logic.

Detection executes a single `docker exec` call that checks all known
agent parent directories:

```bash
docker exec my-container sh -c '
  for d in .claude .cursor .codex .codeium/windsurf .config/agents .roo ...; do
    test -d "$HOME/$d" && echo "$d"
  done
'
```

The output lines are matched back to `AgentKind` values using the
same `default_agent_path()` mapping. For each match, a `TargetConfig`
is generated with `environment = "docker:my-container"`.

### 2.2 Detection Directory List

The detection list MUST be derived from `default_agent_path()` by
stripping the `~/` prefix and the trailing `/skills` segment. This
ensures the detection list stays in sync with `agents.rs` rules.

Example derivation:

| `default_agent_path` | Detection subpath |
| :--- | :--- |
| `~/.claude/skills` | `.claude` |
| `~/.cursor/skills` | `.cursor` |
| `~/.codeium/windsurf/skills` | `.codeium/windsurf` |
| `~/.config/agents/skills` | `.config/agents` |

### 2.3 Container Home Resolution

The detection script uses `$HOME` inside the container. This
correctly resolves regardless of whether the container user is
`root` (`/root/`) or a named user (`/home/user/`).

### 2.4 Fallback

If no agents are detected inside the container, the behavior
mirrors local fallback: default to `ClaudeCode` with a warning:

```text
  ⚠ No installed agents detected in container 'my-container';
    defaulting to claude-code.
```

### 2.5 CLI Integration

When `--target docker:my-container` is specified on `install`:

1. Create a `DockerAdapter` for the container.
2. Call `detect_agents()` to discover installed agents.
3. Use the detected targets (not a hardcoded single target).

This replaces the current `parse_install_target_spec()` behavior
that hardcodes `AgentKind::ClaudeCode`.

### 2.6 skills.toml Representation

Each detected agent generates a separate `[[skills.targets]]` entry:

```toml
[[skills.targets]]
agent = "claude-code"
environment = "docker:my-container"

[[skills.targets]]
agent = "cursor"
environment = "docker:my-container"
```

## 3. Bind Mount Detection

### 3.1 Mechanism

When `DockerAdapter::install()` is called, it MUST first check
whether the target path inside the container is already served by a
bind mount from the host.

Detection uses `docker inspect`:

```bash
docker inspect --format '{{json .Mounts}}' my-container
```

The JSON response contains an array of mount objects:

```json
[
  {
    "Type": "bind",
    "Source": "/home/user/.claude/skills",
    "Destination": "/root/.claude/skills",
    "RW": true,
    "Propagation": "rprivate"
  }
]
```

### 3.2 Match Logic

A bind mount is considered a match when:

1. `Type` == `"bind"`.
2. The install target path starts with (or equals) the `Destination`.
3. The mount is `RW: true` (writable).

When a match is found, the adapter resolves the corresponding host
path:

```
host_path = mount.Source + (target_path - mount.Destination)
```

### 3.3 Install Path Branch

```text
DockerAdapter::install(source, target, mode)
  ├─ bind mount detected for target?
  │   ├─ YES → resolve host_path
  │   │        create symlink/copy on HOST at host_path
  │   │        (same logic as LocalAdapter::install)
  │   └─ NO  → fallback to docker cp (existing behavior)
```

When a bind mount is used, the install mode (symlink/copy) is
honored as-is. The `resolve_install_mode()` forced downgrade to
Copy is skipped.

### 3.4 Uninstall Path Branch

Similarly, `DockerAdapter::uninstall()` MUST check for bind mounts:

- If bind-mounted: remove the target on the HOST filesystem.
- If not: use `docker exec rm -rf` (existing behavior).

## 4. `docker mount-hint` Subcommand

### 4.1 Purpose

Output recommended `-v` / `--mount` flags for a container, helping
users configure bind mounts before starting their container.

### 4.2 Usage

```bash
eden-skills docker mount-hint <container>
```

### 4.3 Output

```text
  Docker mount-hint for container 'my-container':

  Recommended bind mounts (add to your docker run / docker-compose):

    -v ~/.eden-skills/skills:/root/.eden-skills/skills:ro \
    -v ~/.claude/skills:/root/.claude/skills \
    -v ~/.cursor/skills:/root/.cursor/skills

  After adding these mounts, restart the container and run:
    eden-skills apply --target docker:my-container
```

The recommended mounts are derived from:

1. The `storage.root` path in the config (read-only mount).
2. All agent target paths that reference the container in the config.

### 4.4 Container Not Found

If the container does not exist:

```text
  error: container 'xyz' not found
  → Ensure the container exists (docker ps -a).
```

### 4.5 Already Mounted

If all relevant paths are already bind-mounted:

```text
  ✓ Container 'my-container' already has all recommended bind mounts.
```

## 5. Doctor Check

### 5.1 New Finding

`doctor` MUST add a finding for Docker targets without bind mounts:

| Severity | Code | Message |
| :--- | :--- | :--- |
| `info` | `DOCKER_NO_BIND_MOUNT` | `docker target 'my-container' uses copy mode; bind mount recommended for live sync` |

The finding hint MUST suggest running `eden-skills docker mount-hint`.

### 5.2 Scope

This check runs only when the config contains at least one Docker
target. It requires the Docker CLI and a running container; if the
container is not running, the check is skipped silently.

## 6. Install Completion Hint

After `install --target docker:my-container` completes, if the
adapter used `docker cp` (no bind mount detected), a hint line
MUST be appended:

```text
  → Tip: add bind mounts for live sync. Run 'eden-skills docker mount-hint my-container'.
```

## 7. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **DBM-001** | Builder | **P0** | `DockerAdapter::install` MUST check bind mounts via `docker inspect` before copying. | Bind-mount detection logic tested with mock inspect output. |
| **DBM-002** | Builder | **P0** | When bind mount covers the target path, install MUST create symlink/copy on HOST instead of `docker cp`. | Host-side symlink created; no `docker cp` executed. |
| **DBM-003** | Builder | **P0** | `docker mount-hint <container>` subcommand MUST output recommended `-v` flags. | Subcommand produces expected output for configured targets. |
| **DBM-004** | Builder | **P1** | `doctor` MUST report `DOCKER_NO_BIND_MOUNT` for Docker targets without bind mounts. | Finding appears in doctor output. |
| **DBM-005** | Builder | **P1** | Install completion MUST show bind-mount hint when `docker cp` was used. | Hint line present after docker-target install. |
| **DBM-006** | Builder | **P1** | `docs/04-docker-targets.md` MUST be updated with bind-mount usage guide and agent auto-detection. | Documentation covers mount setup and detection. |
| **DBM-007** | Builder | **P0** | `--target docker:<container>` MUST auto-detect agents inside the container via `DockerAdapter::detect_agents()`, replacing the hardcoded ClaudeCode default. | Multiple agents detected; each generates a separate target. |

## 8. Backward Compatibility

| Existing Feature | Phase 2.95 Behavior |
| :--- | :--- |
| `docker cp` install | Still used as fallback when no bind mount detected. |
| `--copy` with docker target | Unchanged; copy is used on host when bind-mounted. |
| `remove --target docker:` | Uses bind mount detection for host-side removal. |
| Containers started without mounts | Fully backward-compatible; `docker cp` as before. |
| `--json` output | Unchanged. |
| Existing `skills.toml` with manual docker targets | Unchanged; manual `agent`+`path`+`environment` entries are honored as-is. Auto-detection only triggers from `--target docker:` CLI flag. |
| `--target docker:` with no agents in container | Falls back to ClaudeCode (same as local no-agent fallback). |
