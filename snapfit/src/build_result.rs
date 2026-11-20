//! Entity: turns what built a fallible field into that field's value or a
//! `BuildError`, for the code `#[container]` generates.

use std::error::Error;

use crate::BuildError;

#[cfg(test)]
mod tests;

/// What builds a field marked `fallible = true`: a `Result` whose error
/// converts into `Box<dyn Error + Send + Sync>`. Generated code calls it;
/// its message is what a user reads when the field's builder returns a
/// plain value instead.
#[diagnostic::on_unimplemented(
    message = "the field is marked `fallible = true`, so what builds it must return a `Result`, not `{Self}`",
    label = "this field is marked `fallible = true`",
    note = "the `Result`'s error must convert into `Box<dyn Error + Send + Sync>`, as any error type, `String` and `anyhow::Error` do"
)]
pub trait BuildResult {
    /// The field's value, when building it succeeded.
    type Value;

    fn into_build_result(
        self,
        container: &'static str,
        field: &'static str,
    ) -> Result<Self::Value, BuildError>;
}

impl<T, E: Into<Box<dyn Error + Send + Sync>>> BuildResult for Result<T, E> {
    type Value = T;

    fn into_build_result(
        self,
        container: &'static str,
        field: &'static str,
    ) -> Result<T, BuildError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(BuildError {
                container,
                field,
                source: error.into(),
            }),
        }
    }
}
