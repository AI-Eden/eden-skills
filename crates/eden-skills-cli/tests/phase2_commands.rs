mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;

use common::{
    create_symlink, eden_command, init_git_repo, path_to_file_url, remove_symlink, toml_escape_path,
};

/// Wrapper: takes test_root and uses test_root.join("home") as HOME (phase2_commands convention).
fn eden_command_from_root(test_root: &Path) -> Command {
    eden_command(&test_root.join("home"))
}

#[test]
fn update_clones_configured_registries() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "skill-origin",
        &[("packages/browser/README.md", "seed")],
    );
    let official_registry = init_git_repo(
        temp.path(),
        "registry-official",
        &[("manifest.toml", "format_version = 1\nname = \"official\"\n")],
    );
    let forge_registry = init_git_repo(
        temp.path(),
        "registry-forge",
        &[("manifest.toml", "format_version = 1\nname = \"forge\"\n")],
    );

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[registries]
official = {{ url = "{official_url}", priority = 100 }}
forge = {{ url = "{forge_url}", priority = 10 }}

[[skills]]
id = "phase1-skill"

[skills.source]
repo = "{skill_url}"
subpath = "packages/browser"
ref = "main"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            official_url = path_to_file_url(&official_registry),
            forge_url = path_to_file_url(&forge_registry),
            skill_url = path_to_file_url(&skill_repo),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let output = eden_command_from_root(temp.path())
        .args(["update", "--config"])
        .arg(&config_path)
        .output()
        .expect("run update");

    assert_eq!(
        output.status.code(),
        Some(0),
        "update should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Update") && stdout.contains("registries synced"),
        "expected update header summary, got: {stdout}"
    );
    assert!(
        stdout.contains("Registry") && stdout.contains("Status") && stdout.contains("Detail"),
        "expected update results table headers, got: {stdout}"
    );
    assert!(
        !stdout.contains("registry sync:"),
        "legacy update summary format should be removed, got: {stdout}"
    );
    assert!(storage_root.join("registries/official/.git").exists());
    assert!(storage_root.join("registries/forge/.git").exists());
    assert!(
        storage_root
            .join("registries/official/.git/shallow")
            .exists(),
        "expected official registry to be shallow-cloned"
    );
    assert!(
        storage_root.join("registries/forge/.git/shallow").exists(),
        "expected forge registry to be shallow-cloned"
    );
    assert!(
        storage_root
            .join("registries/official/.eden-last-sync")
            .exists(),
        "expected official registry sync marker"
    );
    assert!(
        storage_root
            .join("registries/forge/.eden-last-sync")
            .exists(),
        "expected forge registry sync marker"
    );
}

#[test]
fn install_resolves_skill_from_registry_and_persists_mode_b_entry() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "google-search-origin",
        &[("skill/README.md", "google-search")],
    );
    let head = git_head(&skill_repo);

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let registry_cache = storage_root.join("registries").join("official");
    write_registry_index_entry(
        &registry_cache,
        "google-search",
        &path_to_file_url(&skill_repo),
        "1.2.0",
        "main",
        &head,
    );

    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[registries]
official = {{ url = "https://example.com/official.git", priority = 100 }}

[[skills]]
id = "phase1-skill"

[skills.source]
repo = "{skill_url}"
subpath = "skill"
ref = "main"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            skill_url = path_to_file_url(&skill_repo),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let output = eden_command_from_root(temp.path())
        .args(["install", "google-search", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let written = fs::read_to_string(&config_path).expect("read config");
    assert!(
        written.contains("name = \"google-search\""),
        "expected mode b name entry in config, got:\n{written}"
    );
    assert!(
        written.contains("version = \"*\""),
        "expected default version constraint in config, got:\n{written}"
    );
}

#[test]
fn apply_and_repair_resolve_mode_b_skills_before_source_sync() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "google-search-origin",
        &[("skill/README.md", "google-search")],
    );
    let head = git_head(&skill_repo);

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let registry_cache = storage_root.join("registries").join("official");
    write_registry_index_entry(
        &registry_cache,
        "google-search",
        &path_to_file_url(&skill_repo),
        "1.2.0",
        "main",
        &head,
    );

    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[registries]
official = {{ url = "https://example.com/official.git", priority = 100 }}

[[skills]]
name = "google-search"
version = "^1.0"
registry = "official"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let apply_output = eden_command_from_root(temp.path())
        .args(["apply", "--config"])
        .arg(&config_path)
        .output()
        .expect("run apply");
    assert_eq!(
        apply_output.status.code(),
        Some(0),
        "apply should succeed, stderr={}",
        String::from_utf8_lossy(&apply_output.stderr)
    );

    let target_skill = target_root.join("google-search");
    assert!(
        fs::symlink_metadata(&target_skill)
            .expect("target metadata")
            .file_type()
            .is_symlink(),
        "apply should create symlink target for mode b skill"
    );

    remove_symlink(&target_skill).expect("remove target symlink");
    let broken = temp.path().join("broken-link-target");
    create_symlink(&broken, &target_skill).expect("create broken symlink");

    let repair_output = eden_command_from_root(temp.path())
        .args(["repair", "--config"])
        .arg(&config_path)
        .output()
        .expect("run repair");
    assert_eq!(
        repair_output.status.code(),
        Some(0),
        "repair should succeed, stderr={}",
        String::from_utf8_lossy(&repair_output.stderr)
    );
}

#[test]
fn apply_and_repair_accept_concurrency_flag() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "phase1-origin",
        &[("skill/README.md", "phase1-skill")],
    );

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[reactor]
concurrency = 1

[[skills]]
id = "phase1-skill"

[skills.source]
repo = "{skill_url}"
subpath = "skill"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            skill_url = path_to_file_url(&skill_repo),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let apply_output = eden_command_from_root(temp.path())
        .args(["apply", "--concurrency", "1", "--config"])
        .arg(&config_path)
        .output()
        .expect("run apply");
    assert_eq!(
        apply_output.status.code(),
        Some(0),
        "apply should accept --concurrency, stderr={}",
        String::from_utf8_lossy(&apply_output.stderr)
    );

    let repair_output = eden_command_from_root(temp.path())
        .args(["repair", "--concurrency", "1", "--config"])
        .arg(&config_path)
        .output()
        .expect("run repair");
    assert_eq!(
        repair_output.status.code(),
        Some(0),
        "repair should accept --concurrency, stderr={}",
        String::from_utf8_lossy(&repair_output.stderr)
    );
}

#[test]
fn tm_p29_026_apply_and_repair_use_tree_style_install_lines() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "phase1-origin-tree",
        &[("skill/README.md", "phase1-skill")],
    );

    let storage_root = temp.path().join("storage");
    let target_root_a = temp.path().join("agent-target-a");
    let target_root_b = temp.path().join("agent-target-b");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[[skills]]
id = "phase1-skill"

[skills.source]
repo = "{skill_url}"
subpath = "skill"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "custom"
path = "{target_root_a}"

[[skills.targets]]
agent = "custom"
path = "{target_root_b}"
"#,
            storage_root = toml_escape_path(&storage_root),
            skill_url = path_to_file_url(&skill_repo),
            target_root_a = toml_escape_path(&target_root_a),
            target_root_b = toml_escape_path(&target_root_b),
        ),
    )
    .expect("write config");

    let apply_output = eden_command_from_root(temp.path())
        .args(["--color", "never", "apply", "--config"])
        .arg(&config_path)
        .output()
        .expect("run apply");
    assert_eq!(
        apply_output.status.code(),
        Some(0),
        "apply should succeed, stderr={}",
        String::from_utf8_lossy(&apply_output.stderr)
    );
    let apply_stdout = String::from_utf8_lossy(&apply_output.stdout);
    assert!(
        apply_stdout.contains("Install"),
        "apply output should include Install prefix, stdout={apply_stdout}"
    );
    assert!(
        apply_stdout.contains("✓ phase1-skill"),
        "apply output should include grouped skill header, stdout={apply_stdout}"
    );
    assert!(
        apply_stdout.contains("├─") && apply_stdout.contains("└─"),
        "apply output should include tree connectors, stdout={apply_stdout}"
    );
    assert!(
        apply_stdout.contains("(symlink)"),
        "apply output should include mode label in tree lines, stdout={apply_stdout}"
    );
    assert!(
        !apply_stdout.contains("~>"),
        "legacy arrow line format should be removed from apply output, stdout={apply_stdout}"
    );

    let target_skill = target_root_a.join("phase1-skill");
    remove_symlink(&target_skill).expect("remove target symlink");
    let broken = temp.path().join("broken-link-target-tree");
    create_symlink(&broken, &target_skill).expect("create broken symlink");

    let repair_output = eden_command_from_root(temp.path())
        .args(["--color", "never", "repair", "--config"])
        .arg(&config_path)
        .output()
        .expect("run repair");
    assert_eq!(
        repair_output.status.code(),
        Some(0),
        "repair should succeed, stderr={}",
        String::from_utf8_lossy(&repair_output.stderr)
    );
    let repair_stdout = String::from_utf8_lossy(&repair_output.stdout);
    assert!(
        repair_stdout.contains("Install"),
        "repair output should include Install prefix, stdout={repair_stdout}"
    );
    assert!(
        repair_stdout.contains("✓ phase1-skill"),
        "repair output should include grouped skill header, stdout={repair_stdout}"
    );
    assert!(
        repair_stdout.contains("└─"),
        "repair output should include tree-style target connector lines, stdout={repair_stdout}"
    );
    assert!(
        repair_stdout.contains("(symlink)"),
        "repair output should include mode label in tree lines, stdout={repair_stdout}"
    );
    assert!(
        !repair_stdout.contains("~>"),
        "legacy arrow line format should be removed from repair output, stdout={repair_stdout}"
    );
}

#[test]
fn apply_concurrency_flag_overrides_config_value_for_validation() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "phase1-origin",
        &[("skill/README.md", "phase1-skill")],
    );

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[reactor]
concurrency = 1

[[skills]]
id = "phase1-skill"

[skills.source]
repo = "{skill_url}"
subpath = "skill"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            skill_url = path_to_file_url(&skill_repo),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let output = eden_command_from_root(temp.path())
        .args(["apply", "--concurrency", "0", "--config"])
        .arg(&config_path)
        .output()
        .expect("run apply");

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected validation exit code 2"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("INVALID_CONCURRENCY"),
        "expected INVALID_CONCURRENCY error, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn install_dry_run_displays_resolution_without_side_effects() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "google-search-origin",
        &[("skill/README.md", "google-search")],
    );
    let head = git_head(&skill_repo);

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let registry_cache = storage_root.join("registries").join("official");
    write_registry_index_entry(
        &registry_cache,
        "google-search",
        &path_to_file_url(&skill_repo),
        "1.2.0",
        "main",
        &head,
    );

    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[registries]
official = {{ url = "https://example.com/official.git", priority = 100 }}

[[skills]]
id = "phase1-skill"

[skills.source]
repo = "{skill_url}"
subpath = "skill"
ref = "main"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            skill_url = path_to_file_url(&skill_repo),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");
    let original_config = fs::read_to_string(&config_path).expect("read original config");

    let output = eden_command_from_root(temp.path())
        .args(["install", "google-search", "--dry-run", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install dry-run");

    assert_eq!(
        output.status.code(),
        Some(0),
        "install --dry-run should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Dry Run")
            && stdout.contains("Skill / Version / Source")
            && stdout.contains("Install Targets"),
        "expected dry-run preview sections with titles, stdout={stdout}"
    );
    assert!(
        stdout.contains("#")
            && stdout.contains("Skill")
            && stdout.contains("Version")
            && stdout.contains("Source")
            && stdout.contains("Agent")
            && stdout.contains("Path")
            && stdout.contains("Mode"),
        "expected dry-run skill and target tables, stdout={stdout}"
    );
    assert!(
        !stdout.contains("Skill:") && !stdout.contains("Version:") && !stdout.contains("Source:"),
        "legacy dry-run key-value metadata lines must be removed, stdout={stdout}"
    );
    assert!(
        !stdout.contains("target agent="),
        "legacy dry-run key=value target format should be removed, stdout={stdout}"
    );
    let resolved_repo = path_to_file_url(&skill_repo);
    let compact_stdout = stdout
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace() && *ch != '|')
        .collect::<String>();
    assert!(
        compact_stdout.contains(&resolved_repo),
        "expected resolved repo in output (repo={resolved_repo}), stdout={stdout}"
    );

    let after_config = fs::read_to_string(&config_path).expect("read config after dry-run");
    assert_eq!(
        after_config, original_config,
        "install --dry-run must not modify config"
    );
    assert!(
        !storage_root.join("google-search").exists(),
        "install --dry-run must not clone skill source"
    );
    assert!(
        !target_root.join("google-search").exists(),
        "install --dry-run must not mutate target filesystem"
    );
}

#[test]
fn install_fails_with_update_hint_when_registry_cache_missing() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "google-search-origin",
        &[("skill/README.md", "google-search")],
    );

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[registries]
official = {{ url = "https://example.com/official.git", priority = 100 }}

[[skills]]
id = "phase1-skill"

[skills.source]
repo = "{skill_url}"
subpath = "skill"
ref = "main"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            skill_url = path_to_file_url(&skill_repo),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let output = eden_command_from_root(temp.path())
        .args(["install", "google-search", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");

    assert_eq!(
        output.status.code(),
        Some(1),
        "expected runtime failure exit code 1"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Run `eden-skills update` first."),
        "expected actionable update hint, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn install_warns_when_registry_manifest_is_missing() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "google-search-origin",
        &[("skill/README.md", "google-search")],
    );
    let head = git_head(&skill_repo);

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let registry_cache = storage_root.join("registries").join("official");
    write_registry_index_entry_without_manifest(
        &registry_cache,
        "google-search",
        &path_to_file_url(&skill_repo),
        "1.2.0",
        "main",
        &head,
    );

    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[registries]
official = {{ url = "https://example.com/official.git", priority = 100 }}

[[skills]]
id = "phase1-skill"

[skills.source]
repo = "{skill_url}"
subpath = "skill"
ref = "main"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            skill_url = path_to_file_url(&skill_repo),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let output = eden_command_from_root(temp.path())
        .args(["install", "google-search", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "install should still succeed without manifest, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("manifest.toml"),
        "expected warning about missing manifest.toml, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn windows_style_file_url_is_toml_safe_in_test_configs() {
    let windows_like_repo = PathBuf::from(r"C:\Users\ci\repo");
    let document = format!(
        r#"
[skills.source]
repo = "{repo_url}"
"#,
        repo_url = path_to_file_url(&windows_like_repo),
    );

    let parsed = toml::from_str::<toml::Value>(&document);
    assert!(
        parsed.is_ok(),
        "expected windows-style file URL to remain TOML-safe, got: {parsed:?}"
    );
}

fn write_registry_index_entry(
    registry_root: &Path,
    skill_name: &str,
    repo_url: &str,
    version: &str,
    git_ref: &str,
    commit: &str,
) {
    let first = skill_name
        .chars()
        .next()
        .expect("skill name")
        .to_ascii_lowercase();
    let index_dir = registry_root.join("index").join(first.to_string());
    fs::create_dir_all(&index_dir).expect("create index dir");
    fs::write(
        registry_root.join("manifest.toml"),
        "format_version = 1\nname = \"official\"\n",
    )
    .expect("write manifest");

    let body = format!(
        r#"
[skill]
name = "{skill_name}"
repo = "{repo_url}"
subpath = "skill"

[[versions]]
version = "{version}"
ref = "{git_ref}"
commit = "{commit}"
yanked = false
"#
    );
    fs::write(index_dir.join(format!("{skill_name}.toml")), body).expect("write index entry");
}

fn write_registry_index_entry_without_manifest(
    registry_root: &Path,
    skill_name: &str,
    repo_url: &str,
    version: &str,
    git_ref: &str,
    commit: &str,
) {
    let first = skill_name
        .chars()
        .next()
        .expect("skill name")
        .to_ascii_lowercase();
    let index_dir = registry_root.join("index").join(first.to_string());
    fs::create_dir_all(&index_dir).expect("create index dir");

    let body = format!(
        r#"
[skill]
name = "{skill_name}"
repo = "{repo_url}"
subpath = "skill"

[[versions]]
version = "{version}"
ref = "{git_ref}"
commit = "{commit}"
yanked = false
"#
    );
    fs::write(index_dir.join(format!("{skill_name}.toml")), body).expect("write index entry");
}

fn git_head(repo: &Path) -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .expect("read head");
    assert!(
        output.status.success(),
        "git rev-parse failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
