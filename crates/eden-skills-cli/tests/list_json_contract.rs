use std::fs;
use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn list_json_has_required_schema() {
    let temp = tempdir().expect("tempdir");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[storage]
root = "./storage"

[[skills]]
id = "demo"

[skills.source]
repo = "file:///tmp/unused"
subpath = "packages/browser"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "custom"
path = "./targets"

[skills.verify]
enabled = true
checks = ["path-exists", "target-resolves", "is-symlink"]
"#,
    )
    .expect("write config");

    let out = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["list", "--json", "--config"])
        .arg(&config_path)
        .output()
        .expect("run list --json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "list --json should succeed, stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let payload: Value = serde_json::from_slice(&out.stdout).expect("valid json");

    assert!(
        payload.get("count").and_then(|v| v.as_u64()).is_some(),
        "count must be integer"
    );

    let skills = payload
        .get("skills")
        .and_then(|v| v.as_array())
        .expect("skills must be array");
    assert!(!skills.is_empty(), "skills must be non-empty");

    for skill in skills {
        let obj = skill.as_object().expect("skill must be object");
        assert!(obj.get("id").and_then(|v| v.as_str()).is_some());

        let source = obj
            .get("source")
            .and_then(|v| v.as_object())
            .expect("source object");
        for key in ["repo", "ref", "subpath"] {
            assert!(
                source.get(key).and_then(|v| v.as_str()).is_some(),
                "source.{key} must be string"
            );
        }

        let install = obj
            .get("install")
            .and_then(|v| v.as_object())
            .expect("install object");
        let mode = install
            .get("mode")
            .and_then(|v| v.as_str())
            .expect("install.mode string");
        assert!(matches!(mode, "symlink" | "copy"));

        let verify = obj
            .get("verify")
            .and_then(|v| v.as_object())
            .expect("verify object");
        assert!(
            verify.get("enabled").and_then(|v| v.as_bool()).is_some(),
            "verify.enabled must be bool"
        );
        let checks = verify
            .get("checks")
            .and_then(|v| v.as_array())
            .expect("verify.checks array");
        for check in checks {
            assert!(
                check.as_str().is_some(),
                "verify.checks entries must be strings"
            );
        }

        let targets = obj
            .get("targets")
            .and_then(|v| v.as_array())
            .expect("targets array");
        assert!(!targets.is_empty(), "targets must be non-empty");
        for target in targets {
            let t = target.as_object().expect("target must be object");
            assert!(t.get("agent").and_then(|v| v.as_str()).is_some());
            assert!(t.get("path").and_then(|v| v.as_str()).is_some());
        }
    }
}
