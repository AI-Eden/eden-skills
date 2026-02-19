# Quickstart: First Successful Run

This tutorial gets you from zero to a clean `doctor` run.

## Audience

- First-time users
- Local workstation setup (no Docker required)

## Prerequisites

- Rust toolchain (`cargo`)
- Git
- A writable workspace directory

## Step 0: Enter Repository

```bash
git clone <your-repo-url>
cd eden-skills
```

For convenience in this tutorial:

```bash
ES="cargo run -p eden-skills-cli --"
```

You can replace `$ES` with `eden-skills` if you already installed the binary.

## Step 1: Initialize a Config

```bash
$ES init --config ./skills.quickstart.toml
```

Expected result:

- A new file `skills.quickstart.toml` is created.
- It contains one starter skill (`browser-tool`) with sane defaults.

## Step 2: Preview Planned Actions

```bash
$ES plan --config ./skills.quickstart.toml
```

`plan` is read-only.  
Typical actions:

- `create` for missing targets
- `noop` if state already matches
- `conflict` when local state disagrees with config

JSON mode:

```bash
$ES plan --config ./skills.quickstart.toml --json
```

## Step 3: Apply Changes

```bash
$ES apply --config ./skills.quickstart.toml
```

You should see summaries similar to:

- `source sync: cloned=... updated=... skipped=... failed=...`
- `safety summary: permissive=... non_permissive=...`
- `apply summary: create=... update=... noop=... conflict=... skipped_no_exec=...`
- `apply verification: ok`

## Step 4: Diagnose Current State

```bash
$ES doctor --config ./skills.quickstart.toml
```

If everything is healthy, output is:

- `doctor: no issues detected`

Machine-readable diagnostics:

```bash
$ES doctor --config ./skills.quickstart.toml --json
```

## Step 5: Repair Drift (When Needed)

If you manually break a symlink or delete a target, use:

```bash
$ES repair --config ./skills.quickstart.toml
```

`repair` reuses the same planning and verification contracts as `apply`, but focuses on reconciling drift.

## What to Learn Next

- Manage multi-skill configs: `02-config-lifecycle.md`
- Use registry install flow: `03-registry-and-install.md`
