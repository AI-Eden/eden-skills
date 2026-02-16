# SPEC_SCHEMA_EXT.md

Phase 2 extensions to the `skills.toml` schema.

**Base contract:** `spec/phase1/SPEC_SCHEMA.md`
**Rule:** This file defines additive changes only. It MUST NOT contradict or
override Phase 1 base semantics.

## 1. Purpose

Extend the `skills.toml` schema to support registry-based skill resolution,
SemVer version constraints, and multi-environment target declarations.

## 2. New Top-Level Section: `[registries]`

```toml
[registries]
official = { url = "https://github.com/eden-skills/registry-official.git", priority = 100 }
forge    = { url = "https://github.com/eden-skills/registry-forge.git", priority = 10 }
```

### Fields

| Field | Type | Required | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `url` | string | MUST | -- | Git URL of the registry index repository. |
| `priority` | integer | SHOULD | `0` | Resolution priority weight. Higher value = checked first. |
| `auto_update` | boolean | MAY | `false` | If true, `eden apply` triggers implicit `eden update` for this registry. |

### Validation Rules

- `url` MUST be a valid Git URL (same rules as `source.repo` in Phase 1).
- `priority` MUST be a non-negative integer.
- Duplicate registry names MUST fail validation.
- The `[registries]` section is OPTIONAL. When absent, only direct Git source
  skills are supported (Phase 1 backward compatibility).

## 3. Extended `[[skills]]` Fields

Phase 2 adds an alternative skill declaration mode alongside the existing
direct Git source mode:

```toml
# Mode A: Direct Git source (Phase 1 - unchanged)
[[skills]]
id = "my-private-tool"
source = { repo = "https://...", ref = "main" }

# Mode B: Registry resolution (Phase 2 - new)
[[skills]]
name = "google-search"
version = "^2.0"
registry = "official"   # Optional: constrain to a specific registry
```

### New Fields

| Field | Type | Required | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `name` | string | MUST (Mode B) | -- | Skill name for registry lookup. Mutually exclusive with `id` + `source`. |
| `version` | string | SHOULD (Mode B) | `"*"` (latest) | SemVer version constraint. |
| `registry` | string | MAY | -- | Constrain resolution to a specific named registry. |

### Validation Rules

- A skill entry MUST use either Mode A (`id` + `source`) or Mode B (`name`).
  Mixing both in one entry MUST fail validation.
- When Mode B is used, `[registries]` section MUST be defined.
- `version` string MUST be valid SemVer constraint syntax (exact, `^`, `~`, `*`).

## 4. Extended `[[skills.targets]]` Fields

```toml
[[skills.targets]]
environment = "docker:my-agent-container"
path = "/workspace/skills"
```

### New Fields

| Field | Type | Required | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `environment` | string | MAY | `"local"` | Target environment. `"local"` or `"docker:<container_name>"`. |

### Validation Rules

- `environment` MUST match pattern `"local"` or `"docker:<name>"`.
- When `environment` starts with `"docker:"`, the container name part MUST
  be non-empty.
- Phase 1 targets without `environment` field default to `"local"`.

## 5. Backward Compatibility

| Phase 1 Feature | Phase 2 Behavior |
| :--- | :--- |
| `version = 1` in config | Remains valid. No version bump required for Phase 2 extensions. |
| `id` + `source` skill entries | Continue to work as-is. |
| `agent` field in targets | Continue to work. `environment` is additive. |
| No `[registries]` section | Valid config. Registry commands (`update`, `install by name`) are unavailable. |

## 6. Error Contract

Phase 2 schema validation failures MUST follow the same error contract as
Phase 1 (`spec/phase1/SPEC_SCHEMA.md` Section 5):

- Exit code `2`.
- Machine-readable error code + human-readable message.
- Field path in error message.
