# Troubleshooting Guide

This page maps common failures to concrete recovery steps.

## Quick Triage Flow

1. Run diagnostics:

```bash
cargo run -p eden-skills-cli -- doctor --config ./skills.toml
```

2. If drift exists, try:

```bash
cargo run -p eden-skills-cli -- repair --config ./skills.toml
```

3. If source sync failed, inspect the `source sync failed ...` detail line (includes `skill`, `stage`, `repo_dir`, `detail`).

## Common Cases

### A) `source sync failed ... stage=clone|fetch|checkout`

Meaning:

- Git source synchronization failed before mutation.

Fixes:

- Verify repo URL and credentials
- Ensure local filesystem permissions for storage root
- Retry with network connectivity restored

### B) `BROKEN_SYMLINK` / `TARGET_RESOLVE_MISMATCH`

Meaning:

- Target symlink is broken or points to a different source than expected.

Fixes:

```bash
cargo run -p eden-skills-cli -- repair --config ./skills.toml
```

If conflicts persist, inspect:

```bash
cargo run -p eden-skills-cli -- plan --config ./skills.toml
```

### C) `TARGET_NOT_SYMLINK` or copy/symlink mode mismatch

Meaning:

- Existing target type does not match configured install mode.

Fixes:

- Align config (`install.mode`) with desired target type
- Or remove conflicting target path and re-run `apply`

### D) `DOCKER_NOT_FOUND`

Meaning:

- Docker CLI cannot be invoked for Docker targets.

Fixes:

- Install Docker
- Ensure `docker` is on PATH
- Verify optional override `EDEN_SKILLS_DOCKER_BIN` is valid

### E) `ADAPTER_HEALTH_FAIL`

Meaning:

- Docker target container is not running or not healthy.

Fixes:

```bash
docker start <container-name>
```

Then re-run `doctor` or `apply`.

### F) `REGISTRY_STALE`

Meaning:

- Local registry cache missing, old, or marker invalid.

Fix:

```bash
cargo run -p eden-skills-cli -- update --config ./skills.toml
```

### G) `INVALID_SEMVER` / `UNKNOWN_REGISTRY` / `MISSING_REGISTRIES`

Meaning:

- Registry-mode config is invalid.

Fixes:

- Correct version constraint syntax (`*`, exact, `^`, `~`, ranges)
- Ensure `registry = "<name>"` exists under `[registries]`
- Add `[registries]` when using Mode B skills (`name`/`version`)

### H) Windows symlink privilege errors

Symptoms:

- Symlink creation fails with permission denied.

Fixes:

- Enable Windows Developer Mode, or
- Run with Administrator privileges

The runtime error includes this remediation hint.

## JSON Diagnostics for Tooling

Use JSON output in automation:

```bash
cargo run -p eden-skills-cli -- doctor --config ./skills.toml --json > doctor.json
```

`doctor.json` includes stable keys:

- `summary.total`, `summary.error`, `summary.warning`
- `findings[].code`
- `findings[].severity`
- `findings[].skill_id`
- `findings[].target_path`
- `findings[].message`
- `findings[].remediation`

## When to Use `--strict`

Use strict mode when drift must fail fast:

```bash
cargo run -p eden-skills-cli -- doctor --config ./skills.toml --strict
```

Strict mode is ideal for CI policy gates where unresolved issues should block promotion.
