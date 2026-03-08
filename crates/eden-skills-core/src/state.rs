//! Lightweight integrity metadata attached to installed skills.
//!
//! Reserved for future use by the lock-file diffing and `doctor` commands
//! to track the provenance of each installed artifact.

/// Snapshot of source provenance recorded at install time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityMetadata {
    pub repo: String,
    pub commit_sha: String,
    pub retrieved_at: String,
}
