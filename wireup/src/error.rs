use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireupError {
    ResolutionFailed(String),
    MissingDependency(String),
    InitializationFailed(String),
}

impl fmt::Display for WireupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResolutionFailed(reason) => write!(f, "resolution failed: {reason}"),
            Self::MissingDependency(dep) => write!(f, "missing dependency: {dep}"),
            Self::InitializationFailed(reason) => write!(f, "initialization failed: {reason}"),
        }
    }
}

impl std::error::Error for WireupError {}
