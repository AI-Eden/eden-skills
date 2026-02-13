use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum EdenError {
    InvalidArguments(String),
    Validation(String),
    Runtime(String),
}

impl EdenError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::InvalidArguments(_) => 2,
            Self::Validation(_) => 2,
            Self::Runtime(_) => 1,
        }
    }
}

impl Display for EdenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArguments(msg) => write!(f, "invalid arguments: {msg}"),
            Self::Validation(msg) => write!(f, "validation error: {msg}"),
            Self::Runtime(msg) => write!(f, "runtime error: {msg}"),
        }
    }
}

impl std::error::Error for EdenError {}
