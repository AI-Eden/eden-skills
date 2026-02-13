# SPEC_AGENT_PATHS.md

Normative policy for agent detection and install target resolution.

## 1. Supported Agent Targets (Phase 1)

- `claude-code`
- `cursor`
- `custom`

## 2. Default Runtime Paths

When `target.path` is not specified, resolver MUST use:

- `claude-code`: `~/.claude/skills`
- `cursor`: `~/.cursor/skills`

`custom` has no default path and MUST require explicit `target.path`.

## 3. Resolution Precedence

For each target, final path resolution MUST follow:

1. `targets[].path` (explicit per-target path)
2. `targets[].expected_path` (if provided)
3. Built-in default path map (for known agents only)

If none resolves, CLI MUST fail for that target.

## 3.1 Install Path Derivation

- Resolved target path is treated as the agent skill root directory.
- Effective install path MUST be `<resolved_target_root>/<skill_id>`.
- Plan/apply/doctor/repair operations MUST use the effective install path.

## 4. Normalization Rules

- `~` MUST expand to current user home directory.
- Relative path MUST resolve from config file directory.
- Path MUST be normalized (`.`/`..` collapsed) before comparison.
- Symlink path comparisons MUST use canonical target path for verification.

## 5. Agent Detection

Auto-detection SHOULD be lightweight in Phase 1:

- If known default path exists, mark agent as `detected`.
- If known default path does not exist, mark as `not-detected` but still allow `plan`.
- `apply` MAY create missing target directories unless `--no-create-dirs` is set.

## 6. Failure Semantics

Resolver MUST emit explicit reason codes:

- `TARGET_PATH_UNRESOLVED`
- `TARGET_PATH_INVALID`
- `TARGET_AGENT_UNSUPPORTED`
- `TARGET_PERMISSION_DENIED`
