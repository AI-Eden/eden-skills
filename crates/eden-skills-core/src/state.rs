#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityMetadata {
    pub repo: String,
    pub commit_sha: String,
    pub retrieved_at: String,
}
