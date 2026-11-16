//! Error: what makes snapfit refuse a container, each worded as the sentence
//! the compiler shows, which must read well spoken by a screen reader.

mod accumulated;
mod parts;

use thiserror::Error;

use crate::domain::names::DependencyName;

pub(crate) use accumulated::DomainErrors;
pub(crate) use parts::{AttributeProblem, Feature, FunctionRole};

#[cfg(test)]
mod tests;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomainError {
    #[error("a {label} cannot be blank")]
    BlankName { label: &'static str },

    #[error("dependency cycle: {}", Self::cycle_sentence(.start, .through))]
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

    #[error("#[derive(Container)] can only be applied to a struct")]
    NotAStruct,

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
        "field '{dependency}' must be annotated with #[singleton], #[transient], or #[dispatch]"
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

    #[error("dependency '{dependency}' cannot specify both 'constructor' and 'builder'")]
    ConstructorAndBuilder { dependency: DependencyName },

    #[error(
        "{role} \"{written}\" on field '{dependency}' must be written without quotes: {role} = {written}"
    )]
    QuotedFunctionName {
        dependency: DependencyName,
        role: FunctionRole,
        written: String,
    },

    #[error(
        "{role} '{written}' on field '{dependency}' must be a function name, such as {role} = connect"
    )]
    NotAFunctionName {
        dependency: DependencyName,
        role: FunctionRole,
        written: String,
    },

    #[error(
        "container with mode = \"sync\" cannot contain async dependencies: dependency '{dependency}' is async"
    )]
    AsyncDependencyInSyncContainer { dependency: DependencyName },

    #[error("transient dependency '{dependency}' cannot be async because getters are synchronous")]
    AsyncTransient { dependency: DependencyName },

    #[error(
        "field '{dependency}' has a trait-object type, which has no constructor; build it with a builder such as builder = build_{dependency} (unstable feature)"
    )]
    TraitObjectWithoutBuilder { dependency: DependencyName },

    #[error(
        "#[derive(Container)] does not support #[transient] or #[dispatch] fields; use #[container] instead, which can leave field '{dependency}' out of the struct"
    )]
    NeedsStructRewrite { dependency: DependencyName },

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
            | Self::ConstructorAndBuilder { dependency }
            | Self::QuotedFunctionName { dependency, .. }
            | Self::NotAFunctionName { dependency, .. }
            | Self::AsyncDependencyInSyncContainer { dependency }
            | Self::AsyncTransient { dependency }
            | Self::TraitObjectWithoutBuilder { dependency }
            | Self::NeedsStructRewrite { dependency } => Some(dependency),
            Self::RequiresUnstable { feature } | Self::NotYetImplemented { feature } => {
                feature.dependency()
            }
            Self::BlankName { .. }
            | Self::NotAStruct
            | Self::TupleStruct
            | Self::MultipleContainerAttributes
            | Self::ContainerAttribute { .. } => None,
        }
    }

    /// "a depends on b, which depends on c, which depends on a", or "a depends
    /// on itself", so that no arrow has to be read aloud.
    fn cycle_sentence(start: &DependencyName, through: &[DependencyName]) -> String {
        let Some((first, rest)) = through.split_first() else {
            return format!("{start} depends on itself");
        };
        rest.iter()
            .chain([start])
            .fold(format!("{start} depends on {first}"), add_link)
    }
}

fn add_link(sentence: String, name: &DependencyName) -> String {
    format!("{sentence}, which depends on {name}")
}

fn join_names(names: &[DependencyName]) -> String {
    names
        .iter()
        .map(DependencyName::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}
