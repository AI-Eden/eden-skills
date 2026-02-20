use std::fs;
use std::path::PathBuf;

use toml::Value;

#[test]
fn cargo_install_manifest_is_crates_io_ready() {
    let manifest_path = workspace_root().join("crates/eden-skills-cli/Cargo.toml");
    let manifest_text = fs::read_to_string(&manifest_path).expect("read cli Cargo.toml");
    let manifest: Value = toml::from_str(&manifest_text).expect("parse cli Cargo.toml");

    let package = manifest
        .get("package")
        .and_then(Value::as_table)
        .expect("package table");
    assert_eq!(
        package.get("name").and_then(Value::as_str),
        Some("eden-skills"),
        "cargo install target package should be `eden-skills`"
    );

    for field in ["description", "repository", "homepage", "readme"] {
        let value = package
            .get(field)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        assert!(
            !value.is_empty(),
            "package metadata field `{field}` must be non-empty"
        );
    }

    let keywords = package
        .get("keywords")
        .and_then(Value::as_array)
        .expect("keywords array");
    assert!(
        !keywords.is_empty(),
        "keywords must be present for crates.io discoverability"
    );

    let categories = package
        .get("categories")
        .and_then(Value::as_array)
        .expect("categories array");
    assert!(
        !categories.is_empty(),
        "categories must be present for crates.io indexing"
    );

    let bins = manifest
        .get("bin")
        .and_then(Value::as_array)
        .expect("[[bin]] table list");
    assert!(
        bins.iter()
            .any(|bin| bin.get("name").and_then(Value::as_str) == Some("eden-skills")),
        "binary name `eden-skills` must be defined"
    );

    let dependencies = manifest
        .get("dependencies")
        .and_then(Value::as_table)
        .expect("dependencies table");
    let core_dependency = dependencies
        .get("eden-skills-core")
        .and_then(Value::as_table)
        .expect("eden-skills-core dependency table");
    assert!(
        core_dependency
            .get("version")
            .and_then(Value::as_str)
            .is_some(),
        "eden-skills-core dependency must include a version for publishability"
    );

    let path_only_dependencies = dependencies
        .iter()
        .filter_map(|(name, spec)| {
            let table = spec.as_table()?;
            if table.contains_key("path") && !table.contains_key("version") {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert!(
        path_only_dependencies.is_empty(),
        "path-only dependencies block crates.io publishing: {path_only_dependencies:?}"
    );
}

#[test]
fn release_workflow_covers_targets_archives_and_checksums() {
    let workflow_path = workspace_root().join(".github/workflows/release.yml");
    assert!(
        workflow_path.exists(),
        "release workflow should exist at .github/workflows/release.yml"
    );

    let workflow_text = fs::read_to_string(&workflow_path).expect("read release workflow");

    for required in [
        "tags:",
        "v*",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "eden-skills-${VERSION}-${TARGET}",
        "cargo test --workspace",
        "sha256sum",
        "actions/download-artifact@v4",
        "softprops/action-gh-release@v2",
        "eden-skills --help",
        "eden-skills init",
        "eden-skills install vercel-labs/agent-skills --all",
    ] {
        assert!(
            workflow_text.contains(required),
            "release workflow missing required snippet: {required}"
        );
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}
