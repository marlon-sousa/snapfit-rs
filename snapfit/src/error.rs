use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SnapfitError {
    #[error("resolution failed: {0}")]
    ResolutionFailed(String),

    #[error("missing dependency: {0}")]
    MissingDependency(String),

    #[error("initialization failed: {0}")]
    InitializationFailed(String),
}
