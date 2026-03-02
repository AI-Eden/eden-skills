# eden-skills-core

Core domain logic for [`eden-skills`](https://github.com/AI-Eden/eden-skills) — a deterministic skill manager for AI agent environments.

## What's in this crate

| Module | Responsibility |
| --- | --- |
| `config` | Parse and validate `skills.toml` (Phase 1 / 2 / 2.5 schema) |
| `plan` | Build the action graph (`create` / `update` / `noop` / `conflict` / `remove`) |
| `verify` | Post-install verification checks (path-exists, is-symlink, target-resolves) |
| `lock` | Read / write `skills.lock`; diff-driven reconciliation and orphan detection |
| `reactor` | tokio-based bounded-concurrency task executor with two-phase barrier and cancellation |
| `adapter` | `TargetAdapter` trait + `LocalAdapter` + `DockerAdapter` (via `docker` CLI) |
| `registry` | Multi-registry index parsing, priority fallback, and SemVer version resolution |
| `safety` | License detection, risk label scanning, and `.eden-safety.toml` persistence |
| `source` | Git clone / fetch / checkout with deterministic stage diagnostics |
| `source_format` | Source URL parsing: GitHub shorthand, full URL, tree path, local path, registry name |
| `discovery` | `SKILL.md` discovery across standard agent directories and plugin manifests |
| `agents` | Agent directory rules and auto-detection logic |
| `paths` | Tilde expansion, path normalization, and agent default path resolution |
| `error` | Structured error types via `thiserror` with stable exit-code mapping |

## Usage

This crate is the internal library consumed by the `eden-skills` CLI binary. It is not designed as a general-purpose public API, but all modules are `pub` for downstream integration if needed.

For the user-facing CLI and full documentation, see the main repository:

**[https://github.com/AI-Eden/eden-skills](https://github.com/AI-Eden/eden-skills)**

## License

MIT
