# SPEC_SCHEMA_EXT.md

Phase 2 extensions to the `skills.toml` schema.

**Base contract:** `spec/phase1/SPEC_SCHEMA.md`
**Rule:** This file defines additive changes only. It MUST NOT contradict or
override Phase 1 base semantics.

## 1. Purpose

Extend the `skills.toml` schema to support registry-based skill resolution,
SemVer version constraints, multi-environment target declarations, and
async reactor configuration.

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
- When `registry` is specified, it MUST reference a name defined in `[registries]`.
- Mode A and Mode B entries MAY coexist in the same `[[skills]]` array.
- A Mode B `name` MUST NOT collide with any Mode A `id` in the same config
  (duplicate identifier across modes MUST fail validation).

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

## 5. New Top-Level Section: `[reactor]` (Optional)

```toml
[reactor]
concurrency = 10
```

### Fields

| Field | Type | Required | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `concurrency` | integer | MAY | `10` | Maximum number of concurrent source sync operations. Range: 1-100. |

### Validation Rules

- `concurrency` MUST be an integer in range `[1, 100]`.
- The `[reactor]` section is OPTIONAL. When absent, built-in defaults apply.
- CLI `--concurrency` flag overrides the config value when both are present.

## 6. Backward Compatibility

| Phase 1 Feature | Phase 2 Behavior |
| :--- | :--- |
| `version = 1` in config | Remains valid. No version bump required for Phase 2 extensions. |
| `id` + `source` skill entries | Continue to work as-is (Mode A). |
| `agent` field in targets | Continue to work. `environment` is additive. |
| No `[registries]` section | Valid config. Registry commands (`update`, `install by name`) are unavailable. |
| No `[reactor]` section | Valid config. Built-in defaults apply (concurrency = 10). |

## 7. Error Contract

Phase 2 schema validation failures MUST follow the same error contract as
Phase 1 (`spec/phase1/SPEC_SCHEMA.md` Section 5):

- Exit code `2`.
- Machine-readable error code + human-readable message.
- Field path in error message.

Additional Phase 2 validation error codes:

| Code | Condition |
| :--- | :--- |
| `INVALID_SKILL_MODE` | Skill entry mixes Mode A and Mode B fields. |
| `MISSING_REGISTRIES` | Mode B skill used but `[registries]` section absent. |
| `UNKNOWN_REGISTRY` | `registry` field references undefined registry name. |
| `INVALID_SEMVER` | `version` field is not a valid SemVer constraint. |
| `INVALID_ENVIRONMENT` | `environment` field does not match `local` or `docker:<name>`. |
| `INVALID_CONCURRENCY` | `concurrency` value is outside range `[1, 100]`. |
| `DUPLICATE_SKILL_ID` | A Mode B `name` collides with a Mode A `id` (or duplicate `name`/`id` within the same mode). |

## 8. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **SCH-P2-001** | Builder | **P0** | `[registries]` section with `url`, `priority`, optional `auto_update`. | Config with `[registries]` parses and validates correctly. |
| **SCH-P2-002** | Builder | **P0** | Mode B skill entries (`name` + `version` + optional `registry`). | Config with Mode B entry validates; resolution triggers registry lookup. |
| **SCH-P2-003** | Builder | **P0** | `environment` field in targets (`local`, `docker:<name>`). | Config with `environment = "docker:test"` parses and maps to DockerAdapter. |
| **SCH-P2-004** | Builder | **P0** | Backward compatibility: Phase 1 configs remain valid without changes. | Unmodified Phase 1 config file loads and validates in Phase 2 CLI. |
| **SCH-P2-005** | Builder | **P1** | `[reactor]` section with `concurrency` field (optional, default 10). | Config with `[reactor] concurrency = 5` applies to Reactor behavior. |
| **SCH-P2-006** | Builder | **P0** | Phase 2 validation errors MUST use stable error codes (Section 7 table). | Each error condition produces the documented error code. |

## 9. Resolved Design Decisions (Stage B)

| ID | Item | Decision | Rationale |
| :--- | :--- | :--- | :--- |
| **FC-S1** | Config `version` bump | **Keep `version = 1`**. No bump for Phase 2. | Phase 2 extensions are additive. Phase 1 configs remain valid without changes. The `version` field indicates schema compatibility, not feature level. Since no breaking changes exist, no bump is needed. |
| **FC-S2** | `auto_update` behavior | **Stale-based trigger**: when `auto_update = true` for a registry AND the local index was last synced > 7 days ago, `apply` triggers implicit `update` for that registry only. If network fails during auto-update, proceed with stale cache and emit a warning. | Always updating before `apply` is slow and network-dependent. Stale-based trigger balances freshness with performance. The 7-day threshold aligns with `REGISTRY_STALE` doctor finding (FC-REG3). |
| **FC-S3** | Mode B `id` generation | **Auto-generate `id` from `name`**: the `name` field doubles as the skill identifier for deduplication and diagnostics. If both a Mode A skill with `id = "foo"` and a Mode B skill with `name = "foo"` exist, validation MUST fail (duplicate identifier). | Requiring explicit `id` for Mode B is redundant UX friction. The `name` already uniquely identifies the skill (it is the registry lookup key). Using `name` as the implicit `id` keeps config clean. |
