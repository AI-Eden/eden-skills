use eden_skills_core::config::InstallMode;
use eden_skills_core::plan::{Action, PlanItem};
use serde_json::json;

#[test]
fn plan_json_serialization_keeps_stable_fields() {
    let item = PlanItem {
        skill_id: "demo-skill".to_string(),
        source_path: "/tmp/source".to_string(),
        target_path: "/tmp/target".to_string(),
        install_mode: InstallMode::Symlink,
        action: Action::Create,
        reasons: vec!["target path does not exist".to_string()],
    };

    let payload = serde_json::to_value(vec![item]).expect("serialize plan json");
    let first = payload[0].as_object().expect("plan entry object");

    assert_eq!(first.get("skill_id"), Some(&json!("demo-skill")));
    assert_eq!(first.get("install_mode"), Some(&json!("symlink")));
    assert_eq!(first.get("action"), Some(&json!("create")));
    assert_eq!(
        first.get("reasons"),
        Some(&json!(["target path does not exist"]))
    );
}
