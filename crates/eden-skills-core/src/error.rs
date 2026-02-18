use thiserror::Error;

#[derive(Debug, Error)]
pub enum EdenError {
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("state conflict: {0}")]
    Conflict(String),
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ReactorError {
    #[error("reactor concurrency must be between {min} and {max}, got {provided}")]
    InvalidConcurrency {
        provided: usize,
        min: usize,
        max: usize,
    },
    #[error("reactor runtime initialization failed: {detail}")]
    RuntimeInitialization { detail: String },
    #[error("reactor runtime shutdown: {context}")]
    RuntimeShutdown { context: String },
    #[error("reactor task join failure ({context}): {detail}")]
    TaskJoin { context: String, detail: String },
    #[error("reactor blocking task `{task}` was cancelled")]
    BlockingTaskCancelled { task: String },
    #[error("reactor blocking task `{task}` panicked: {detail}")]
    BlockingTaskPanicked { task: String, detail: String },
    #[error("reactor phase-b failure: {detail}")]
    PhaseB { detail: String },
    #[error("reactor configuration error: {detail}")]
    Config { detail: String },
    #[error("reactor io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("adapter configuration error: {detail}")]
    Config { detail: String },
    #[error("adapter runtime error: {detail}")]
    Runtime { detail: String },
    #[error("adapter io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("registry configuration error: {detail}")]
    Config { detail: String },
    #[error("registry resolution error: {detail}")]
    Resolution { detail: String },
    #[error("registry io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<ReactorError> for EdenError {
    fn from(value: ReactorError) -> Self {
        EdenError::Runtime(value.to_string())
    }
}

impl From<AdapterError> for EdenError {
    fn from(value: AdapterError) -> Self {
        EdenError::Runtime(value.to_string())
    }
}

impl From<RegistryError> for EdenError {
    fn from(value: RegistryError) -> Self {
        EdenError::Runtime(value.to_string())
    }
}
