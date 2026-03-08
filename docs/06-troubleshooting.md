# Troubleshooting Guide

This page maps common failures to concrete recovery steps.

## Quick Triage Flow

1. Run diagnostics:

```bash
CONFIG="${HOME}/.eden-skills/skills.toml"
eden-skills doctor --config "$CONFIG"
```

1. If drift exists, try:

```bash
eden-skills repair --config "$CONFIG"
```

1. If source sync failed, inspect the `source sync failed ...` error line and the `~>` hint for guidance.

## Common Cases

### A) `source sync failed for N repo(s)` (clone / fetch / checkout)

Meaning:

- Git source synchronization failed before mutation. The error message
  extracts the root cause from git stderr (e.g. `repository not found`,
  `authentication failed`, `could not resolve host`).

Fixes:

- Verify repo URL spelling and ensure the repository exists
- Check git credentials and SSH key configuration
- Ensure network connectivity and DNS resolution
- For ref-not-found errors, verify the branch/tag exists in the remote

### B) `BROKEN_SYMLINK` / `TARGET_RESOLVE_MISMATCH`

Meaning:

- Target symlink is broken or points to a different source than expected.

Fixes:

```bash
eden-skills repair --config "$CONFIG"
```

If conflicts persist, inspect:

```bash
eden-skills plan --config "$CONFIG"
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
eden-skills update --config "$CONFIG"
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

### I) Corrupted `skills.lock` Warning During `apply`

Meaning:

- Lock file content is invalid or unsupported, so lock-aware reconciliation falls back to full reconciliation.

Fix:

- Re-run `apply`; the lock file is regenerated automatically on success.
- If needed, inspect or remove the corrupted `skills.lock` manually before retry.

### J) `remove` with No IDs Fails in Non-TTY Context

Meaning:

- Interactive no-argument remove is only available on TTY sessions because the
  command now uses a checkbox selector.

Fixes:

- Provide explicit skill IDs in non-TTY mode:

```bash
eden-skills remove --config "$CONFIG" skill-a skill-b
```

- Or run in an interactive terminal when using no-argument selection.

### K) `ORPHAN_CACHE_ENTRY` or growing `.repos/` cache usage

Meaning:

- Repo-cache directories under `storage/.repos/` are no longer referenced by
  your current config.
- Interrupted install discovery can also leave behind stale
  `eden-skills-discovery-*` temp directories.

Fixes:

```bash
eden-skills clean --config "$CONFIG"
```

If you want cleanup to happen automatically after uninstalling skills:

```bash
eden-skills remove --config "$CONFIG" skill-a --auto-clean
```

### L) `DOCKER_OWNERSHIP_CHANGED`

Meaning:

- A Docker-managed skill was reinstalled locally inside the container, so the
  `.eden-managed` manifest now marks it as `source: "local"`.

Fixes:

- Reclaim ownership from the host side:

```bash
eden-skills apply --config "$CONFIG" --force
```

- Or accept the local takeover and remove the host-side config entry:

```bash
eden-skills remove --config "$CONFIG" <skill-id>
```

### M) `DOCKER_EXTERNALLY_REMOVED`

Meaning:

- A Docker-managed skill was deleted from the container outside the normal
  host-driven remove flow.

Fixes:

- Reinstall it from config:

```bash
eden-skills apply --config "$CONFIG"
```

- Or accept the deletion:

```bash
eden-skills remove --config "$CONFIG" <skill-id>
```

## JSON Diagnostics for Tooling

Use JSON output in automation:

```bash
eden-skills doctor --config "$CONFIG" --json > doctor.json
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
eden-skills doctor --config "$CONFIG" --strict
```

Strict mode is ideal for CI policy gates where unresolved issues should block promotion.
