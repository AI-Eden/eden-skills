# Safety, Strict Mode, and Exit Codes

This guide explains execution safety and automation semantics.

## 1) Safety Gate Basics

During `apply` and `repair`, `eden-skills` analyzes each skill repository and writes:

- `<storage_root>/<skill_id>/.eden-safety.toml`

Metadata includes:

- `license_status` (`permissive`, `non-permissive`, `unknown`)
- `risk_labels` (scripts, executable bits on Unix, binary artifact signatures)
- `commit_sha` (when available)
- `no_exec_metadata_only`

## 2) No-Exec Metadata-Only Mode

Set in config:

```toml
[skills.safety]
no_exec_metadata_only = true
```

Behavior:

- Source sync still runs
- Safety metadata is still written
- Install target mutation is skipped for that skill
- Verification checks are skipped for that skill

Useful for high-risk or license-uncertain skills you want to inventory without executing.

## 3) Strict Mode (`--strict`)

Strict mode changes failure semantics, not payload format.

Examples:

- `doctor --strict`: if findings exist, exits with strict conflict code
- `apply --strict` / `repair --strict`: conflicts become hard failures

Important precedence:

- Source sync runtime failures still exit as runtime error (`1`) before strict conflict handling.

## 4) Exit Code Contract

- `0`: success
- `1`: runtime failure (IO, git/source sync, execution failures)
- `2`: invalid args or config/schema validation failure
- `3`: strict-mode conflict/drift failure

This contract is stable and intended for CI/CD automation.

## 5) Doctor Safety Findings

`doctor` can emit:

- `NO_EXEC_METADATA_ONLY`
- `LICENSE_NON_PERMISSIVE`
- `LICENSE_UNKNOWN`
- `RISK_REVIEW_REQUIRED`

Tip:

```bash
ES="cargo run -p eden-skills-cli --"
CONFIG="${HOME}/.eden-skills/skills.toml"
$ES doctor --config "$CONFIG" --json
```

Use JSON mode for machine parsing and policy gates.

## 6) Suggested Automation Pattern

For CI quality checks:

```bash
set -e
$ES plan --config "$CONFIG" --json > plan.json
$ES doctor --config "$CONFIG" --json > doctor.json
```

For strict policy enforcement:

```bash
$ES doctor --config "$CONFIG" --strict
```

Non-zero exits can be directly consumed by pipeline gates.
