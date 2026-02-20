# SPEC_DISTRIBUTION.md

Binary distribution strategy for `eden-skills`.

## 1. Purpose

Enable users to install `eden-skills` without requiring a Rust toolchain.
Provide prebuilt binaries for major platforms and a frictionless installation
experience.

## 2. Distribution Channels

### 2.1 `cargo install` (Rust Users)

The `eden-skills-cli` crate MUST be publishable to `crates.io` and
installable via:

```bash
cargo install eden-skills
```

Requirements:

- `crates/eden-skills-cli/Cargo.toml` MUST include complete `[package]`
  metadata: `name`, `version`, `edition`, `license`, `description`,
  `repository`, `homepage`, `readme`, `keywords`, `categories`.
- The binary name MUST be `eden-skills`.
- All workspace dependencies MUST resolve from `crates.io` (no
  path-only dependencies that would block publishing).

### 2.2 GitHub Releases (Prebuilt Binaries)

A GitHub Actions release workflow MUST produce prebuilt binaries for:

| Target | Archive | Binary Name |
| :--- | :--- | :--- |
| `x86_64-unknown-linux-gnu` | `.tar.gz` | `eden-skills` |
| `aarch64-unknown-linux-gnu` | `.tar.gz` | `eden-skills` |
| `x86_64-apple-darwin` | `.tar.gz` | `eden-skills` |
| `aarch64-apple-darwin` | `.tar.gz` | `eden-skills` |
| `x86_64-pc-windows-msvc` | `.zip` | `eden-skills.exe` |

Archive naming convention: `eden-skills-{version}-{target}.{ext}`

Example: `eden-skills-0.1.0-x86_64-unknown-linux-gnu.tar.gz`

### 2.3 Shell Installer (Optional)

A convenience install script MAY be provided:

```bash
curl -fsSL https://raw.githubusercontent.com/AI-Eden/eden-skills/main/install.sh | sh
```

The script SHOULD:

- Detect OS and architecture.
- Download the appropriate binary from GitHub Releases.
- Place it in a standard location (`~/.local/bin/` or `/usr/local/bin/`).
- Verify the download via checksum (SHA-256).

## 3. Release Workflow

### 3.1 Trigger

The release workflow MUST trigger on Git tag push matching `v*` (e.g., `v0.1.0`).

### 3.2 Steps

1. Validate tag version matches `Cargo.toml` version.
2. Run full test suite (`cargo test --workspace`) on all target platforms.
3. Build release binaries (`cargo build --release`) for each target.
4. Create archives with appropriate format (`.tar.gz` for Unix, `.zip` for Windows).
5. Generate SHA-256 checksums for all archives.
6. Create a GitHub Release with:
   - Tag name as release title.
   - Auto-generated changelog from commits since last tag.
   - All binary archives and checksums as release assets.

### 3.3 Cross-Compilation

For `aarch64-unknown-linux-gnu`, the workflow SHOULD use `cross` or
a dedicated ARM runner. For macOS targets, native GitHub Actions runners
are available for both x86_64 and aarch64.

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **DST-001** | Builder | **P0** | CLI MUST be installable via `cargo install eden-skills`. | `cargo install eden-skills` from crates.io produces a working binary. |
| **DST-002** | Builder | **P0** | GitHub Actions MUST produce release binaries for all listed targets. | Tag push triggers workflow; all 5 target binaries are attached to the release. |
| **DST-003** | Builder | **P1** | Release archives MUST include SHA-256 checksums. | Checksum file is present in release assets and matches binary hashes. |

## 5. Future Scope (Not in Phase 2.5)

- Homebrew formula (`brew install eden-skills`).
- AUR package for Arch Linux.
- npm wrapper package (`npx eden-skills`).
- Automatic `crates.io` publishing in release workflow.
- Code signing for macOS and Windows binaries.
