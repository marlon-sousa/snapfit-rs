//! Error: what makes snapfit refuse a container, each worded as the sentence
//! the compiler shows, which must read well spoken by a screen reader.

mod accumulated;
mod parts;
mod sentences;

use thiserror::Error;

use crate::domain::names::DependencyName;

pub(crate) use accumulated::DomainErrors;
pub(crate) use parts::{AttributeProblem, Feature};
use sentences::{cycle_sentence, join_names};

#[cfg(test)]
mod tests;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomainError {
    #[error("a {label} cannot be blank")]
    BlankName { label: &'static str },

    #[error("dependency cycle: {}", cycle_sentence(.start, .through))]
    DependencyCycle {
        start: DependencyName,
        /// The dependencies between `start` and its return to `start`.
        through: Vec<DependencyName>,
    },

    #[error(
        "unknown dependency '{}' on field '{dependency}'. You must choose between the dependencies the container can wire: [{}]",
        join_names(.missing),
        join_names(.available)
    )]
    UnknownPrerequisites {
        dependency: DependencyName,
        missing: Vec<DependencyName>,
        available: Vec<DependencyName>,
    },

    #[error("duplicate dependency name: '{0}'")]
    DuplicateDependency(DependencyName),

    #[error("tuple structs are not supported by #[container]; use named fields")]
    TupleStruct,

    #[error("multiple #[container] attributes are not allowed")]
    MultipleContainerAttributes,

    #[error("{problem}")]
    ContainerAttribute { problem: AttributeProblem },

    #[error("field '{dependency}': {problem}")]
    FieldAttribute {
        dependency: DependencyName,
        problem: AttributeProblem,
    },

    #[error(
        "field '{dependency}' has no snapfit attribute; add #[singleton] to have the container build it"
    )]
    Unannotated { dependency: DependencyName },

    #[error("field '{dependency}' cannot declare multiple lifecycle attributes")]
    MultipleLifecycles { dependency: DependencyName },

    #[error("field '{dependency}' cannot declare multiple #[dispatch] attributes")]
    MultipleDispatches { dependency: DependencyName },

    #[error(
        "field '{dependency}' cannot use #[cfg] or #[cfg_attr]; conditional dependencies are not supported"
    )]
    ConditionalField { dependency: DependencyName },

    #[error(
        "build = {written} on field '{dependency}' is not a way to build a field; write build = default, build = factory, or build = constructor({written}) to call a function of that name"
    )]
    UnknownBuild {
        dependency: DependencyName,
        written: String,
    },

    #[error(
        "build \"{written}\" on field '{dependency}' must be written without quotes: build = {written}"
    )]
    QuotedBuild {
        dependency: DependencyName,
        written: String,
    },

    #[error(
        "build = {written} on field '{dependency}' is not a way to build a field; write build = default, build = factory, or build = constructor(connect)"
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
        "leaf_build = {written} cannot build the container's fields that have no deps; write leaf_build = default, or leaf_build = constructor with the function's name in brackets, such as leaf_build = constructor(open)"
    )]
    InvalidLeafBuild { written: String },

    #[error(
        "field '{dependency}' is built with build = default, which takes no dependencies; remove its deps, or build it another way"
    )]
    DefaultWithDependencies { dependency: DependencyName },

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
        "container with mode = \"sync\" cannot contain async dependencies: dependency '{dependency}' is async"
    )]
    AsyncDependencyInSyncContainer { dependency: DependencyName },

    #[error("transient dependency '{dependency}' cannot be async because getters are synchronous")]
    AsyncTransient { dependency: DependencyName },

    #[error(
        "async dependency '{dependency}' cannot be built with build = default, because Default::default() cannot be awaited"
    )]
    AsyncDefault { dependency: DependencyName },

    #[error(
        "field '{dependency}' has a trait-object type, which has no constructor; add build = factory to the field and build it in the build_{} method of the container's factories trait",
        .dependency.without_raw_prefix()
    )]
    TraitObjectWithoutFactory { dependency: DependencyName },

    #[error(
        "{feature} is unstable in this version of snapfit; enable the `unstable` feature of the `snapfit` crate to use it"
    )]
    RequiresUnstable { feature: Feature },

    #[error("{feature} is planned for a future version of snapfit and is not implemented yet")]
    NotYetImplemented { feature: Feature },
}

impl DomainError {
    /// The dependency the error is about, whose field the diagnostic points at.
    pub(crate) fn subject(&self) -> Option<&DependencyName> {
        match self {
            Self::DependencyCycle { start, .. } => Some(start),
            Self::UnknownPrerequisites { dependency, .. }
            | Self::DuplicateDependency(dependency)
            | Self::FieldAttribute { dependency, .. }
            | Self::Unannotated { dependency }
            | Self::MultipleLifecycles { dependency }
            | Self::MultipleDispatches { dependency }
            | Self::ConditionalField { dependency }
            | Self::UnknownBuild { dependency, .. }
            | Self::QuotedBuild { dependency, .. }
            | Self::NotABuild { dependency, .. }
            | Self::ConstructorWithoutName { dependency }
            | Self::DefaultWithDependencies { dependency }
            | Self::QuotedFunctionName { dependency, .. }
            | Self::NotAFunctionName { dependency, .. }
            | Self::AsyncDependencyInSyncContainer { dependency }
            | Self::AsyncTransient { dependency }
            | Self::AsyncDefault { dependency }
            | Self::TraitObjectWithoutFactory { dependency } => Some(dependency),
            Self::RequiresUnstable { feature } | Self::NotYetImplemented { feature } => {
                feature.dependency()
            }
            Self::BlankName { .. }
            | Self::TupleStruct
            | Self::MultipleContainerAttributes
            | Self::InvalidLeafBuild { .. }
            | Self::ContainerAttribute { .. } => None,
        }
    }
}
