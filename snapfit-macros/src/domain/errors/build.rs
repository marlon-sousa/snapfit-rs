//! Error: why a field's build, or the container's leaf_build, cannot build it.

use derive_more::Display;
use thiserror::Error;

use crate::domain::names::DependencyName;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum BuildProblem {
    #[error(
        "build = {written} on field '{dependency}' is not a way to build a field; write build = default, build = factory, build = builder, or build = constructor({written}) to call a function of that name"
    )]
    Unknown {
        dependency: DependencyName,
        written: String,
    },
    #[error(
        "build \"{written}\" on field '{dependency}' must be written without quotes: build = {written}"
    )]
    Quoted {
        dependency: DependencyName,
        written: String,
    },
    #[error(
        "build = {written} on field '{dependency}' is not a way to build a field; write build = default, build = factory, build = builder, or build = constructor(connect)"
    )]
    NotABuild {
        dependency: DependencyName,
        written: String,
    },
    #[error(
        "build = constructor on field '{dependency}' needs the function's name, such as build = constructor(connect)"
    )]
    ConstructorWithoutName { dependency: DependencyName },
    #[error(
        "constructor \"{written}\" on field '{dependency}' must be written without quotes: build = constructor({written})"
    )]
    QuotedFunctionName {
        dependency: DependencyName,
        written: String,
    },
    #[error(
        "constructor '{written}' on field '{dependency}' must be a function name, such as build = constructor(connect)"
    )]
    NotAFunctionName {
        dependency: DependencyName,
        written: String,
    },
    #[error(
        "leaf_build = {written} cannot build the container's fields that have no deps; write leaf_build = default, or leaf_build = constructor with the function's name in brackets, such as leaf_build = constructor(open)"
    )]
    InvalidLeafBuild { written: String },
    #[error(
        "field '{dependency}' is built with build = default, which takes no dependencies; remove its deps, or build it another way"
    )]
    DefaultWithDependencies { dependency: DependencyName },
    #[error("field '{dependency}' is built with {build}")]
    CannotFail {
        dependency: DependencyName,
        build: InfallibleBuild,
    },
}

/// What would build a field marked `fallible = true` that cannot fail, with
/// the way out its error offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub(crate) enum InfallibleBuild {
    #[display(
        "build = default, which cannot fail; remove fallible = true, or build it another way"
    )]
    Default,
    #[display(
        "the container's leaf_build, since it has no deps and no build of its own, and leaf_build cannot fail; give the field a build of its own, such as build = constructor(new), or remove fallible = true"
    )]
    LeafBuild,
}

impl BuildProblem {
    pub(crate) fn subject(&self) -> Option<&DependencyName> {
        match self {
            Self::Unknown { dependency, .. }
            | Self::Quoted { dependency, .. }
            | Self::NotABuild { dependency, .. }
            | Self::ConstructorWithoutName { dependency }
            | Self::QuotedFunctionName { dependency, .. }
            | Self::NotAFunctionName { dependency, .. }
            | Self::DefaultWithDependencies { dependency }
            | Self::CannotFail { dependency, .. } => Some(dependency),
            Self::InvalidLeafBuild { .. } => None,
        }
    }
}
