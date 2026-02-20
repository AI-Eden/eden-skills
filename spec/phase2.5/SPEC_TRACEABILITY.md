# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.5.
Use this file to recover accurate context after compression.

## 1. Install URL Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| MVP-001 | `SPEC_INSTALL_URL.md` 3.1 | `install` MUST accept GitHub shorthand (`owner/repo`) | -- | -- | planned |
| MVP-002 | `SPEC_INSTALL_URL.md` 3.1 | `install` MUST accept full GitHub/GitLab HTTPS URLs | -- | -- | planned |
| MVP-003 | `SPEC_INSTALL_URL.md` 3.4 | `install` MUST accept GitHub tree URLs and extract repo, ref, subpath | -- | -- | planned |
| MVP-004 | `SPEC_INSTALL_URL.md` 3.1 | `install` MUST accept Git SSH URLs | -- | -- | planned |
| MVP-005 | `SPEC_INSTALL_URL.md` 3.1 | `install` MUST accept local paths | -- | -- | planned |
| MVP-006 | `SPEC_INSTALL_URL.md` 3.2 | Source format detection MUST follow documented precedence | -- | -- | planned |
| MVP-007 | `SPEC_INSTALL_URL.md` 3.3 | Skill ID MUST be auto-derived with `--id` override | -- | -- | planned |
| MVP-008 | `SPEC_INSTALL_URL.md` 6.2 | `install` MUST auto-create config if not exists | -- | -- | planned |
| MVP-009 | `SPEC_INSTALL_URL.md` 4.2 | `install` MUST discover SKILL.md in standard directories | -- | -- | planned |
| MVP-010 | `SPEC_INSTALL_URL.md` 5.6 | `--list` MUST display discovered skills without installing | -- | -- | planned |
| MVP-011 | `SPEC_INSTALL_URL.md` 5.1 | `--all` MUST install all discovered skills without confirmation | -- | -- | planned |
| MVP-012 | `SPEC_INSTALL_URL.md` 5.2 | `--skill` MUST install only named skills | -- | -- | planned |
| MVP-013 | `SPEC_INSTALL_URL.md` 5.3 | Interactive mode MUST show skills and prompt for confirmation | -- | -- | planned |
| MVP-014 | `SPEC_INSTALL_URL.md` 5.5 | Non-TTY MUST default to `--all` behavior | -- | -- | planned |
| MVP-015 | `SPEC_INSTALL_URL.md` 4.3 | Single-skill repos MUST skip confirmation | -- | -- | planned |

## 2. Schema Amendment Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| SCH-P25-001 | `SPEC_SCHEMA_P25.md` 2.2 | `skills` array MAY be empty | `crates/eden-skills-core/src/config.rs` (`RawConfig::into_config`, `validate_config`) | `crates/eden-skills-core/tests/config_tests.rs`, `crates/eden-skills-cli/tests/phase25_schema_tests.rs` | completed |
| SCH-P25-002 | `SPEC_SCHEMA_P25.md` 3.2 | `init` template MUST produce minimal config without dummy skills | `crates/eden-skills-cli/src/commands.rs` (`default_config_template`) | `crates/eden-skills-cli/tests/init_command.rs` | completed |
| SCH-P25-003 | `SPEC_SCHEMA_P25.md` 5 | Phase 1 and Phase 2 configs remain valid | `crates/eden-skills-core/src/config.rs` (backward-compatible non-empty skill validation path retained) | `crates/eden-skills-core/tests/config_tests.rs`, `cargo test --workspace` regression gate | completed |

## 3. Agent Detection Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| AGT-001 | `SPEC_AGENT_DETECT.md` 2.2 | `install` MUST auto-detect installed agents | -- | -- | planned |
| AGT-002 | `SPEC_AGENT_DETECT.md` 2.1 | Detection MUST check documented agent directories | -- | -- | planned |
| AGT-003 | `SPEC_AGENT_DETECT.md` 3 | Explicit `--target` MUST override auto-detection | -- | -- | planned |
| AGT-004 | `SPEC_AGENT_DETECT.md` 2.3 | No agents detected MUST fall back to claude-code with warning | -- | -- | planned |

## 4. CLI UX Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| UX-001 | `SPEC_CLI_UX.md` 4 | CLI MUST use colored output with status symbols | -- | -- | planned |
| UX-002 | `SPEC_CLI_UX.md` 6 | CLI MUST use spinner for network operations | -- | -- | planned |
| UX-003 | `SPEC_CLI_UX.md` 4.1 | CLI MUST use `✓`/`✗`/`·`/`!` symbols for results | -- | -- | planned |
| UX-004 | `SPEC_CLI_UX.md` 3.4 | CLI MUST respect `NO_COLOR`/`FORCE_COLOR`/`CI` env vars | -- | -- | planned |
| UX-005 | `SPEC_CLI_UX.md` 3.3 | `--json` output MUST remain identical to Phase 1/2 | -- | -- | planned |
| UX-006 | `SPEC_CLI_UX.md` 3.2 | Non-TTY MUST disable colors and spinners | -- | -- | planned |
| UX-007 | `SPEC_CLI_UX.md` 5.1 | Interactive prompts MUST use `dialoguer` | -- | -- | planned |

## 5. Distribution Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| DST-001 | `SPEC_DISTRIBUTION.md` 2.1 | CLI MUST be installable via `cargo install eden-skills` | -- | -- | planned |
| DST-002 | `SPEC_DISTRIBUTION.md` 2.2 | GitHub Actions MUST produce release binaries | -- | -- | planned |
| DST-003 | `SPEC_DISTRIBUTION.md` 3.2 | Release archives MUST include SHA-256 checksums | -- | -- | planned |

## 6. Test Matrix Coverage

| SCENARIO_ID | Source | Scenario | Automated Test | Status |
|---|---|---|---|---|
| TM-P25-001 | `SPEC_TEST_MATRIX.md` 2 | Empty skills array validation | `crates/eden-skills-core/tests/config_tests.rs` (`load_valid_config_when_skills_array_is_missing`, `load_valid_config_when_skills_array_is_explicitly_empty`) | completed |
| TM-P25-002 | `SPEC_TEST_MATRIX.md` 2 | Empty config plan | `crates/eden-skills-cli/tests/phase25_schema_tests.rs` (`plan_with_empty_config_succeeds_and_reports_zero_actions`, `plan_json_with_empty_config_emits_empty_array`) | completed |
| TM-P25-003 | `SPEC_TEST_MATRIX.md` 2 | Empty config apply | `crates/eden-skills-cli/tests/phase25_schema_tests.rs` (`apply_with_empty_config_succeeds_with_zero_summary`) | completed |
| TM-P25-004 | `SPEC_TEST_MATRIX.md` 2 | Init template minimal | `crates/eden-skills-cli/tests/init_command.rs` (`init_creates_config_when_missing`, `init_overwrites_when_force_is_set`) | completed |
| TM-P25-005 | `SPEC_TEST_MATRIX.md` 2 | Backward compatibility | `crates/eden-skills-core/tests/config_tests.rs` (`load_phase1_style_config_with_five_skills_for_backward_compatibility`), `cargo test --workspace` | completed |
| TM-P25-006 | `SPEC_TEST_MATRIX.md` 3 | GitHub shorthand | -- | planned |
| TM-P25-007 | `SPEC_TEST_MATRIX.md` 3 | Full GitHub URL | -- | planned |
| TM-P25-008 | `SPEC_TEST_MATRIX.md` 3 | GitHub tree URL | -- | planned |
| TM-P25-009 | `SPEC_TEST_MATRIX.md` 3 | Git SSH URL | -- | planned |
| TM-P25-010 | `SPEC_TEST_MATRIX.md` 3 | Local path | -- | planned |
| TM-P25-011 | `SPEC_TEST_MATRIX.md` 3 | Source format precedence | -- | planned |
| TM-P25-012 | `SPEC_TEST_MATRIX.md` 3 | Registry fallback | -- | planned |
| TM-P25-013 | `SPEC_TEST_MATRIX.md` 4 | Auto-derived ID | -- | planned |
| TM-P25-014 | `SPEC_TEST_MATRIX.md` 4 | ID override | -- | planned |
| TM-P25-015 | `SPEC_TEST_MATRIX.md` 4 | ID upsert | -- | planned |
| TM-P25-016 | `SPEC_TEST_MATRIX.md` 5 | Single SKILL.md at root | -- | planned |
| TM-P25-017 | `SPEC_TEST_MATRIX.md` 5 | Multiple skills in `skills/` | -- | planned |
| TM-P25-018 | `SPEC_TEST_MATRIX.md` 5 | Multiple skills in `packages/` | -- | planned |
| TM-P25-019 | `SPEC_TEST_MATRIX.md` 5 | No SKILL.md found | -- | planned |
| TM-P25-020 | `SPEC_TEST_MATRIX.md` 5 | List flag | -- | planned |
| TM-P25-021 | `SPEC_TEST_MATRIX.md` 6 | Install all flag | -- | planned |
| TM-P25-022 | `SPEC_TEST_MATRIX.md` 6 | Install specific skills | -- | planned |
| TM-P25-023 | `SPEC_TEST_MATRIX.md` 6 | Unknown skill name | -- | planned |
| TM-P25-024 | `SPEC_TEST_MATRIX.md` 6 | Interactive confirmation (TTY) | -- | planned |
| TM-P25-025 | `SPEC_TEST_MATRIX.md` 6 | Non-TTY default | -- | planned |
| TM-P25-026 | `SPEC_TEST_MATRIX.md` 7 | Multi-agent detection | -- | planned |
| TM-P25-027 | `SPEC_TEST_MATRIX.md` 7 | No agent fallback | -- | planned |
| TM-P25-028 | `SPEC_TEST_MATRIX.md` 7 | Target override | -- | planned |
| TM-P25-029 | `SPEC_TEST_MATRIX.md` 8 | Fresh system install | -- | planned |
| TM-P25-030 | `SPEC_TEST_MATRIX.md` 8 | Missing parent directory | -- | planned |
| TM-P25-031 | `SPEC_TEST_MATRIX.md` 9 | TTY color output | -- | planned |
| TM-P25-032 | `SPEC_TEST_MATRIX.md` 9 | NO_COLOR compliance | -- | planned |
| TM-P25-033 | `SPEC_TEST_MATRIX.md` 9 | JSON mode unchanged | -- | planned |
| TM-P25-034 | `SPEC_TEST_MATRIX.md` 9 | Spinner during clone | -- | planned |
| TM-P25-035 | `SPEC_TEST_MATRIX.md` 10 | Cargo install | -- | planned |
| TM-P25-036 | `SPEC_TEST_MATRIX.md` 10 | Release binary | -- | planned |
