//! Error: why a fallible container could not be built, naming the container
//! and the field that failed, and keeping that field's own error.

use std::error::Error;

use thiserror::Error;

#[cfg(test)]
mod tests;

/// The error `try_new()` returns when a field marked `fallible = true` fails
/// to build.
///
/// Its message names the container and the field, and the error the field's
/// constructor, builder or factory returned is kept whole as its
/// [`source`](Error::source), so that it can be logged, or downcast to its
/// own type when it is an error type.
#[derive(Error, Debug)]
#[error("snapfit could not build the field `{field}` of `{container}`")]
#[non_exhaustive]
pub struct BuildError {
    pub(crate) container: &'static str,
    pub(crate) field: &'static str,
    pub(crate) source: Box<dyn Error + Send + Sync>,
}

impl BuildError {
    /// The name of the container that was being built.
    pub fn container(&self) -> &str {
        self.container
    }

    /// The name of the field whose constructor, builder or factory failed.
    pub fn field(&self) -> &str {
        self.field
    }
}
