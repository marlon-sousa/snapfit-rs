use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum WireupError {
    #[error("resolution failed: {0}")]
    ResolutionFailed(String),

    #[error("missing dependency: {0}")]
    MissingDependency(String),

    #[error("initialization failed: {0}")]
    InitializationFailed(String),

    #[error("unknown dependency key: {0}")]
    UnknownKey(String),
}
