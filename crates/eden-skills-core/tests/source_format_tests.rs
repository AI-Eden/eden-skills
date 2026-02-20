use std::path::Path;

use eden_skills_core::source_format::{
    derive_skill_id_from_source_repo, detect_install_source, DetectedInstallSource, UrlSourceType,
};

#[test]
fn github_shorthand_expands_to_https_git_url() {
    let detected =
        detect_install_source("vercel-labs/agent-skills", Path::new(".")).expect("detect source");
    let url_source = match detected {
        DetectedInstallSource::Url(source) => source,
        DetectedInstallSource::RegistryName(_) => panic!("expected url source"),
    };

    assert_eq!(url_source.source_type, UrlSourceType::GitHubShorthand);
    assert_eq!(
        url_source.repo,
        "https://github.com/vercel-labs/agent-skills.git"
    );
}

#[test]
fn full_github_url_appends_git_suffix_when_missing() {
    let detected =
        detect_install_source("https://github.com/user/repo", Path::new(".")).expect("detect");
    let url_source = match detected {
        DetectedInstallSource::Url(source) => source,
        DetectedInstallSource::RegistryName(_) => panic!("expected url source"),
    };

    assert_eq!(url_source.source_type, UrlSourceType::FullUrl);
    assert_eq!(url_source.repo, "https://github.com/user/repo.git");
}

#[test]
fn github_tree_url_extracts_repo_ref_and_subpath() {
    let detected = detect_install_source(
        "https://github.com/owner/repo/tree/main/skills/browser",
        Path::new("."),
    )
    .expect("detect");
    let url_source = match detected {
        DetectedInstallSource::Url(source) => source,
        DetectedInstallSource::RegistryName(_) => panic!("expected url source"),
    };

    assert_eq!(url_source.source_type, UrlSourceType::GitHubTree);
    assert_eq!(url_source.repo, "https://github.com/owner/repo.git");
    assert_eq!(url_source.reference.as_deref(), Some("main"));
    assert_eq!(url_source.subpath.as_deref(), Some("skills/browser"));
}

#[test]
fn git_ssh_url_is_accepted_as_url_mode() {
    let detected =
        detect_install_source("git@github.com:user/repo.git", Path::new(".")).expect("detect");
    let url_source = match detected {
        DetectedInstallSource::Url(source) => source,
        DetectedInstallSource::RegistryName(_) => panic!("expected url source"),
    };

    assert_eq!(url_source.source_type, UrlSourceType::SshUrl);
    assert_eq!(url_source.repo, "git@github.com:user/repo.git");
}

#[test]
fn local_path_is_detected_before_shorthand() {
    let detected = detect_install_source("./owner/repo", Path::new(".")).expect("detect");
    let url_source = match detected {
        DetectedInstallSource::Url(source) => source,
        DetectedInstallSource::RegistryName(_) => panic!("expected url source"),
    };

    assert_eq!(url_source.source_type, UrlSourceType::LocalPath);
    assert!(url_source.is_local);
}

#[test]
fn unmatched_source_falls_back_to_registry_name() {
    let detected = detect_install_source("browser-tool", Path::new(".")).expect("detect");
    match detected {
        DetectedInstallSource::RegistryName(name) => assert_eq!(name, "browser-tool"),
        DetectedInstallSource::Url(_) => panic!("expected registry fallback"),
    }
}

#[test]
fn auto_derived_id_uses_repo_tail_without_git_suffix() {
    let derived = derive_skill_id_from_source_repo("https://github.com/user/my-skill.git")
        .expect("derive id");
    assert_eq!(derived, "my-skill");
}
