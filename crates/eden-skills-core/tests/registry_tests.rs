use std::fs;
use std::path::Path;

use eden_skills_core::registry::{
    parse_registry_specs_from_toml, resolve_skill_from_registry_sources,
    sort_registry_specs_by_priority, RegistrySource,
};
use tempfile::tempdir;

#[test]
fn parse_registry_specs_supports_multiple_entries_with_priorities() {
    let config_toml = r#"
[registries]
official = { url = "https://example.com/official.git", priority = 100 }
forge = { url = "https://example.com/forge.git", priority = 10 }
"#;

    let specs = parse_registry_specs_from_toml(config_toml).expect("parse registry specs");
    assert_eq!(specs.len(), 2);

    let ordered = sort_registry_specs_by_priority(&specs);
    assert_eq!(ordered[0].name, "official");
    assert_eq!(ordered[0].priority, 100);
    assert_eq!(ordered[1].name, "forge");
    assert_eq!(ordered[1].priority, 10);
}

#[test]
fn resolve_skill_uses_priority_fallback_order() {
    let temp = tempdir().expect("tempdir");
    let official_root = temp.path().join("official");
    let forge_root = temp.path().join("forge");

    write_index_entry(
        &official_root,
        "browser-use",
        "https://example.com/official/browser-use.git",
        &[
            (
                "1.2.0",
                "v1.2.0",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                false,
            ),
            (
                "1.0.0",
                "v1.0.0",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                false,
            ),
        ],
    );
    write_index_entry(
        &forge_root,
        "browser-use",
        "https://example.com/forge/browser-use.git",
        &[(
            "2.0.0",
            "v2.0.0",
            "cccccccccccccccccccccccccccccccccccccccc",
            false,
        )],
    );

    let sources = vec![
        RegistrySource {
            name: "forge".to_string(),
            priority: 10,
            root: forge_root,
        },
        RegistrySource {
            name: "official".to_string(),
            priority: 100,
            root: official_root,
        },
    ];

    let resolved =
        resolve_skill_from_registry_sources(&sources, "browser-use", None).expect("resolve");

    assert_eq!(resolved.registry_name, "official");
    assert_eq!(
        resolved.repo,
        "https://example.com/official/browser-use.git"
    );
    assert_eq!(resolved.version, "1.2.0");
}

#[test]
fn resolve_skill_matches_semver_constraints() {
    let temp = tempdir().expect("tempdir");
    let official_root = temp.path().join("official");

    write_index_entry(
        &official_root,
        "semver-demo",
        "https://example.com/official/semver-demo.git",
        &[
            (
                "1.2.0",
                "v1.2.0",
                "1111111111111111111111111111111111111111",
                false,
            ),
            (
                "1.2.3",
                "v1.2.3",
                "2222222222222222222222222222222222222222",
                false,
            ),
            (
                "1.2.5",
                "v1.2.5",
                "3333333333333333333333333333333333333333",
                false,
            ),
            (
                "1.9.9",
                "v1.9.9",
                "4444444444444444444444444444444444444444",
                false,
            ),
            (
                "2.0.0",
                "v2.0.0",
                "5555555555555555555555555555555555555555",
                false,
            ),
            (
                "3.0.0",
                "v3.0.0",
                "6666666666666666666666666666666666666666",
                true,
            ),
        ],
    );

    let sources = vec![RegistrySource {
        name: "official".to_string(),
        priority: 100,
        root: official_root,
    }];

    let exact = resolve_skill_from_registry_sources(&sources, "semver-demo", Some("1.2.3"))
        .expect("resolve exact version");
    assert_eq!(exact.version, "1.2.3");

    let caret =
        resolve_skill_from_registry_sources(&sources, "semver-demo", Some("^1.2")).expect("caret");
    assert_eq!(caret.version, "1.9.9");

    let tilde = resolve_skill_from_registry_sources(&sources, "semver-demo", Some("~1.2.3"))
        .expect("tilde");
    assert_eq!(tilde.version, "1.2.5");

    let range = resolve_skill_from_registry_sources(&sources, "semver-demo", Some(">=1.0,<2.0"))
        .expect("range");
    assert_eq!(range.version, "1.9.9");

    let wildcard =
        resolve_skill_from_registry_sources(&sources, "semver-demo", Some("*")).expect("wildcard");
    assert_eq!(wildcard.version, "2.0.0");
}

fn write_index_entry(
    registry_root: &Path,
    skill_name: &str,
    repo: &str,
    versions: &[(&str, &str, &str, bool)],
) {
    let first = skill_name
        .chars()
        .next()
        .expect("skill name should not be empty")
        .to_ascii_lowercase();
    let index_dir = registry_root.join("index").join(first.to_string());
    fs::create_dir_all(&index_dir).expect("create index dir");

    let mut body = String::new();
    body.push_str("[skill]\n");
    body.push_str(&format!("name = \"{skill_name}\"\n"));
    body.push_str("description = \"test skill\"\n");
    body.push_str(&format!("repo = \"{repo}\"\n"));
    body.push_str("subpath = \".\"\n");
    body.push_str("license = \"MIT\"\n\n");

    for (version, git_ref, commit, yanked) in versions {
        body.push_str("[[versions]]\n");
        body.push_str(&format!("version = \"{version}\"\n"));
        body.push_str(&format!("ref = \"{git_ref}\"\n"));
        body.push_str(&format!("commit = \"{commit}\"\n"));
        body.push_str(&format!("yanked = {yanked}\n\n"));
    }

    fs::write(index_dir.join(format!("{skill_name}.toml")), body).expect("write index entry");
}
