# Docker Targets Tutorial

This guide explains Docker-target configuration, diagnostics, and adapter contracts.

## Audience

- Users running agents inside Docker containers
- CI environments validating Docker target behavior

## Target Model

For `targets[].environment` in `skills.toml`:

- `environment = "local"`: local target semantics
- `environment = "docker:<container>"`: Docker-target semantics (container-aware checks and adapter flows)

## Example Config

```toml
[[skills]]
id = "browser-tool"

[skills.source]
repo = "https://github.com/vercel-labs/skills.git"
subpath = "packages/browser"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "custom"
path = "/workspace/agent-skills"
environment = "docker:my-agent"

[skills.verify]
enabled = true
checks = ["path-exists", "content-present"]

[skills.safety]
no_exec_metadata_only = false
```

## Current CLI Focus for Docker Targets

At CLI level, Docker target coverage is currently strongest in:

- `doctor` Docker health diagnostics
- adapter-backed uninstall path used by `remove`
- config validation for `environment = "docker:<container>"`

Example diagnostics:

```bash
ES="cargo run -p eden-skills-cli --"
$ES doctor --config ./skills.toml
```

## Adapter Contract: Symlink Fallback

At adapter contract level (`DockerAdapter`), if `install.mode = "symlink"` is requested:

- A warning is emitted
- Effective mode falls back to copy

This is expected because host/container boundaries do not support safe cross-boundary symlink behavior.

## Diagnose Docker Issues

Relevant finding codes:

- `DOCKER_NOT_FOUND`
- `ADAPTER_HEALTH_FAIL`

## Typical Fixes

- `DOCKER_NOT_FOUND`:
  - Install Docker
  - Ensure Docker CLI is available in PATH

- `ADAPTER_HEALTH_FAIL`:
  - Start container (`docker start <container>`)
  - Verify daemon access and container name

## Remove Flow with Docker Targets

When using:

```bash
$ES remove --config ./skills.toml <skill-id>
```

the CLI uses adapter uninstall semantics for each target.  
For Docker targets, uninstall is performed through container commands.

## Optional: Force a Custom Docker Binary (Testing)

For deterministic CI/test scenarios:

```bash
EDEN_SKILLS_DOCKER_BIN=/path/to/docker $ES doctor --config ./skills.toml
```

If the binary path is invalid, `doctor` will report `DOCKER_NOT_FOUND`.
