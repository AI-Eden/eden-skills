# eden-skills Tutorials

This directory contains task-oriented guides for features implemented in Phase 1, Phase 2, and Phase 2.7 closeout scope.

If you are new, read in order.

## Learning Path

1. [Quickstart: First Successful Run](01-quickstart.md)  
   First run with `init`, `plan`, `apply`, `doctor`, `repair`.

2. [Config Lifecycle Management](02-config-lifecycle.md)  
   Manage `skills.toml` from CLI (`add`, `remove`, `set`, `list`, `config export/import`).

3. [Registry and Install Workflow](03-registry-and-install.md)  
   Use Phase 2 registry workflow (`update`, `install`) and Mode A/Mode B config.

4. [Docker Targets Guide](04-docker-targets.md)  
   Configure Docker targets and understand Docker adapter behavior.

5. [Safety, Strict Mode, and Exit Codes](05-safety-strict-and-exit-codes.md)  
   Safety metadata, strict mode semantics, and automation-friendly exit codes.

6. [Troubleshooting Playbook](06-troubleshooting.md)  
   Diagnose common failures using finding codes and command output patterns.

## Phase 2.7 Coverage Map

- `skills.lock` lifecycle and reconciliation behavior: `01-quickstart.md`, `02-config-lifecycle.md`, `06-troubleshooting.md`
- Help and output polish (`--version`, `-V`, `--color`, improved errors): `01-quickstart.md`, `05-safety-strict-and-exit-codes.md`
- Remove enhancements (batch remove, interactive remove, `-y`): `02-config-lifecycle.md`, `06-troubleshooting.md`

## Command Contract References

For normative behavior, refer to specs:

- [Phase 1 Command Contract](../spec/phase1/SPEC_COMMANDS.md)
- [Phase 1 Schema Contract](../spec/phase1/SPEC_SCHEMA.md)
- [Phase 2 Command Extensions](../spec/phase2/SPEC_COMMANDS_EXT.md)
- [Phase 2 Schema Extensions](../spec/phase2/SPEC_SCHEMA_EXT.md)
- [Phase 2 Adapter Contract](../spec/phase2/SPEC_ADAPTER.md)
- [Phase 2 Registry Contract](../spec/phase2/SPEC_REGISTRY.md)
- [Phase 2.5 Install URL Contract](../spec/phase2.5/SPEC_INSTALL_URL.md)
- [Phase 2.7 Lock Contract](../spec/phase2.7/SPEC_LOCK.md)
- [Phase 2.7 Help Contract](../spec/phase2.7/SPEC_HELP_SYSTEM.md)
- [Phase 2.7 Output Contract](../spec/phase2.7/SPEC_OUTPUT_POLISH.md)
- [Phase 2.7 Remove Contract](../spec/phase2.7/SPEC_REMOVE_ENH.md)

For current implementation state:

- [Status Snapshot](../STATUS.yaml)
- [Execution Tracker](../EXECUTION_TRACKER.md)
