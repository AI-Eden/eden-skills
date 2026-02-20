# eden-skills

Deterministic skill installation and reconciliation for agent environments.

`eden-skills` is a local Rust CLI that keeps skill state predictable across tools like Claude Code and Cursor.  
It manages the full lifecycle from planning and apply to diagnostics, repair, registry resolution, and Docker-aware target health checks.

## Current Status

- Phase 1 (CLI foundation): complete
- Phase 2 (Hyper-Loop core): complete
- Cross-platform CI (Linux/macOS/Windows): passing
- Phase 3 (crawler/taxonomy/curation engine): not implemented yet

### Development Notice

`eden-skills` is still under active development.

- Please avoid using it in production environments for now unless you can tolerate breaking changes and evolving behavior.
- Community contributions are very welcome (issues, bug reports, docs, tests, and pull requests).
- If you want to contribute, please align changes with the spec-first workflow in [`spec/`](spec/) and track updates in [`STATUS.yaml`](STATUS.yaml) / [`EXECUTION_TRACKER.md`](EXECUTION_TRACKER.md).

Authoritative status files:

- [Status Snapshot](STATUS.yaml) (machine-readable status)
- [Execution Tracker](EXECUTION_TRACKER.md) (execution and ownership log)
- [Roadmap](ROADMAP.md) (strategic milestones)

## What You Can Do Today

- Reconcile local skill state with deterministic `plan` / `apply` / `doctor` / `repair`
- Manage config from CLI (`init`, `list`, `add`, `remove`, `set`, `config export/import`)
- Resolve and install registry skills (`update`, `install`)
- Configure Docker targets (`environment = "docker:<container>"`) with adapter-backed health diagnostics and uninstall flows
- Use bounded async concurrency for `apply` / `repair` / `update`
- Enforce safety metadata and risk signals (`.eden-safety.toml`, license/risk findings, metadata-only mode)

## Quick Start

Prerequisites:

- Rust toolchain (`cargo`)
- Git
- Docker (optional, only for Docker targets)

Install from source (recommended):

```bash
git clone https://github.com/AI-Eden/eden-skills.git
cd eden-skills

# Install to ~/.cargo/bin/eden-skills
cargo install --path crates/eden-skills-cli --locked --force

# Verify install
eden-skills install --help
```

If `eden-skills` is not found, add Cargo bin to your `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Run without installing (development mode):

```bash
# From repository root: /path/to/eden-skills

# Initialize a demo config in current directory
cargo run -p eden-skills -- init --config ./skills.demo.toml

# Dry-run the action graph
cargo run -p eden-skills -- plan --config ./skills.demo.toml

# Apply changes
cargo run -p eden-skills -- apply --config ./skills.demo.toml

# Diagnose drift or risk findings
cargo run -p eden-skills -- doctor --config ./skills.demo.toml
```

For a complete, production-style walkthrough, start with:

- [Quickstart Tutorial](docs/01-quickstart.md)

## Documentation

Read the [Tutorial Index](docs/README.md) for the full learning path.

- [Quickstart: First Successful Run](docs/01-quickstart.md) - first run (`init` to `doctor`)
- [Config Lifecycle Management](docs/02-config-lifecycle.md) - manage skills via CLI (`add/remove/set/list/config`)
- [Registry and Install Workflow](docs/03-registry-and-install.md) - Phase 2 registry workflow (`update` + `install`)
- [Docker Targets Guide](docs/04-docker-targets.md) - Docker target configuration and behavior
- [Safety, Strict Mode, and Exit Codes](docs/05-safety-strict-and-exit-codes.md) - safety model, strict mode, and exit semantics
- [Troubleshooting Playbook](docs/06-troubleshooting.md) - common failures and recovery playbook

## Command Surface

Primary commands:

- Core reconciliation: `plan`, `apply`, `doctor`, `repair`
- Registry workflow: `update`, `install`
- Lifecycle/config: `init`, `list`, `add`, `remove`, `set`, `config export`, `config import`

Global patterns:

- `--config <path>`: custom config path (default: `~/.eden-skills/skills.toml`)
- `--strict`: convert drift/warnings into strict failure semantics
- `--json`: machine-readable output for automation
- `--concurrency <n>`: override reactor concurrency on `apply`, `repair`, `update`

## Exit Codes

- `0`: success
- `1`: runtime failure
- `2`: config/schema/argument validation failure
- `3`: strict-mode conflict/drift failure

## Repository Layout

- [`crates/eden-skills-core`](crates/eden-skills-core): domain logic (config, plan, verify, safety, reactor, adapter, registry)
- [`crates/eden-skills-cli`](crates/eden-skills-cli): user-facing CLI binary (`eden-skills`)
- [`crates/eden-skills-indexer`](crates/eden-skills-indexer): indexer entrypoint placeholder for future phase
- [`spec/`](spec/): normative behavior contracts (Phase 1 frozen + Phase 2 extensions)
- [`docs/`](docs/): user tutorials and operational guides

## Spec-First Contract

Behavior is defined in [`spec/`](spec/) first, then implemented in code.

- [Spec Index](spec/README.md)
- [Phase 1 Contracts](spec/phase1/)
- [Phase 2 Contracts](spec/phase2/)
- Requirement traceability: [Phase 1](spec/phase1/SPEC_TRACEABILITY.md), [Phase 2](spec/phase2/SPEC_TRACEABILITY.md)

## Future Scope

Phase 3 platform capabilities (crawler, taxonomy, curation rubric) are tracked but not yet implemented.  
See [Roadmap](ROADMAP.md) for strategic milestones and [Execution Tracker](EXECUTION_TRACKER.md) for ownership boundaries.
