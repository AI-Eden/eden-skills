# Docker Targets Tutorial

This guide explains the Docker-target workflow in Phase 2.95: container agent auto-detection, bind-mount-aware installs, `docker mount-hint`, and related diagnostics.

## Target Model

For `targets[].environment` in `skills.toml`:

- `environment = "local"` keeps normal host installs.
- `environment = "docker:<container>"` tells eden-skills to install into a running Docker container.

## Auto-Detect During Install

When you install directly to a Docker target:

```bash
CONFIG="${HOME}/.eden-skills/skills.toml"
eden-skills install https://github.com/vercel-labs/agent-skills.git --target docker:my-agent --config "$CONFIG"
```

the CLI now:

1. Connects to the running container.
2. Detects installed agents from the container's `$HOME` (for example `.claude`, `.cursor`, `.codex`, `.codeium/windsurf`).
3. Writes one `[[skills.targets]]` entry per detected agent.

Example persisted config:

```toml
[[skills.targets]]
agent = "claude-code"
environment = "docker:my-agent"

[[skills.targets]]
agent = "cursor"
environment = "docker:my-agent"
```

If no supported agent directories are detected in the container, install falls back to `claude-code` and prints a warning.

Existing manual Docker targets are preserved as-is. Auto-detection only triggers when you explicitly use `--target docker:<container>`.

## Bind Mount Behavior

Before copying files into a container, `DockerAdapter` inspects the container mounts:

- If the target path is covered by a writable bind mount, eden-skills installs on the host path behind that mount.
- If no writable bind mount covers the target path, eden-skills falls back to `docker cp`.

This has two important effects:

- Host-side bind mounts allow live sync without repeated `docker cp`.
- `install.mode = "symlink"` is honored when the target is bind-mounted, because the symlink is created on the host filesystem.

Without a bind mount, Docker installs still work, but symlink mode falls back to copy and install prints a follow-up hint:

```text
  → Tip: add bind mounts for live sync. Run 'eden-skills docker mount-hint my-agent'.
```

## Generate Recommended Mounts

Use the dedicated helper command to print the bind mounts you should add to `docker run` or `docker-compose`:

```bash
eden-skills docker mount-hint my-agent --config "$CONFIG"
```

The output includes:

- A read-only mount for `storage.root` into the container's `~/.eden-skills/skills`
- Writable mounts for every configured Docker agent target that references that container

Example output:

```text
  Docker mount-hint for container 'my-agent':

  Recommended bind mounts (add to your docker run / docker-compose):

    -v /home/me/.eden-skills/skills:/root/.eden-skills/skills:ro
    -v /home/me/.claude/skills:/root/.claude/skills
    -v /home/me/.cursor/skills:/root/.cursor/skills

  After adding these mounts, restart the container and run:
    eden-skills apply --config "$CONFIG"
```

If all recommended mounts already exist, the command reports that the container is already fully covered.

## Doctor Findings

Relevant Docker-related `doctor` findings now include:

- `DOCKER_NOT_FOUND`: Docker CLI is unavailable.
- `ADAPTER_HEALTH_FAIL`: container is missing or not running.
- `DOCKER_NO_BIND_MOUNT`: Docker target is running in copy mode without a writable bind mount.

For the last case, the remediation points to:

```bash
eden-skills docker mount-hint <container>
```

## Remove / Uninstall Semantics

Docker uninstall now follows the same bind-mount-aware logic:

- If the Docker target path is backed by a host bind mount, eden-skills removes the host-side target.
- Otherwise it falls back to `docker exec rm -rf ...` inside the container.

This applies both to direct adapter uninstall flows and to higher-level cleanup paths that rely on Docker target removal.

## Optional: Override Docker Binary for Tests

For deterministic CI/test scenarios:

```bash
EDEN_SKILLS_DOCKER_BIN=/path/to/docker eden-skills doctor --config "$CONFIG"
```

If the binary path is invalid, `doctor` reports `DOCKER_NOT_FOUND`.
