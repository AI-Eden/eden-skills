use std::fs;

use eden_skills_core::discovery::discover_skills;
use tempfile::tempdir;

#[test]
fn discovers_single_root_skill_markdown() {
    let temp = tempdir().expect("tempdir");
    fs::write(
        temp.path().join("SKILL.md"),
        r#"---
name: root-skill
description: Root skill description
---
"#,
    )
    .expect("write skill");

    let discovered = discover_skills(temp.path()).expect("discover");
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].name, "root-skill");
    assert_eq!(discovered[0].description, "Root skill description");
    assert_eq!(discovered[0].subpath, ".");
}

#[test]
fn discovers_multiple_skills_under_skills_directory() {
    let temp = tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("skills/a")).expect("mkdir");
    fs::create_dir_all(temp.path().join("skills/b")).expect("mkdir");
    fs::write(
        temp.path().join("skills/a/SKILL.md"),
        r#"---
name: skill-a
description: A
---
"#,
    )
    .expect("write a");
    fs::write(
        temp.path().join("skills/b/SKILL.md"),
        r#"---
name: skill-b
description: B
---
"#,
    )
    .expect("write b");

    let discovered = discover_skills(temp.path()).expect("discover");
    assert_eq!(discovered.len(), 2);
    assert_eq!(discovered[0].name, "skill-a");
    assert_eq!(discovered[0].subpath, "skills/a");
    assert_eq!(discovered[1].name, "skill-b");
    assert_eq!(discovered[1].subpath, "skills/b");
}

#[test]
fn discovers_multiple_skills_under_packages_directory() {
    let temp = tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("packages/x")).expect("mkdir");
    fs::create_dir_all(temp.path().join("packages/y")).expect("mkdir");
    fs::write(
        temp.path().join("packages/x/SKILL.md"),
        r#"---
name: pkg-x
description: X
---
"#,
    )
    .expect("write x");
    fs::write(
        temp.path().join("packages/y/SKILL.md"),
        r#"---
name: pkg-y
description: Y
---
"#,
    )
    .expect("write y");

    let discovered = discover_skills(temp.path()).expect("discover");
    assert_eq!(discovered.len(), 2);
    assert_eq!(discovered[0].name, "pkg-x");
    assert_eq!(discovered[0].subpath, "packages/x");
    assert_eq!(discovered[1].name, "pkg-y");
    assert_eq!(discovered[1].subpath, "packages/y");
}

#[test]
fn returns_empty_when_no_skill_markdown_exists() {
    let temp = tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("skills/a")).expect("mkdir");
    fs::write(temp.path().join("skills/a/README.md"), "no skill").expect("write readme");

    let discovered = discover_skills(temp.path()).expect("discover");
    assert!(discovered.is_empty());
}
