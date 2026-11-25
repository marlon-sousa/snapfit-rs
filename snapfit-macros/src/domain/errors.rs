//! Error: what makes snapfit refuse a container or a dispatched trait, each
//! worded as the sentence the compiler shows, which must read well spoken by
//! a screen reader.

mod accumulated;
mod asynchrony;
mod dispatched_trait;
mod parts;
mod sentences;
mod subject;
mod trait_objects;

use thiserror::Error;

use crate::domain::names::{DependencyName, SetterName};

pub(crate) use accumulated::DomainErrors;
pub(crate) use asynchrony::AsyncProblem;
pub(crate) use dispatched_trait::{DispatchProblem, ItemProblem};
pub(crate) use parts::{AttributeProblem, Feature, InfallibleBuild};
use sentences::{cycle_sentence, dependencies_noun, join_names};
pub(crate) use subject::DispatchSubject;
pub(crate) use trait_objects::TraitObjectProblem;

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
        "unknown {} '{}' on field '{dependency}'. You must choose between the dependencies the container can wire: [{}]",
        dependencies_noun(.missing),
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
        "build = {written} on field '{dependency}' is not a way to build a field; write build = default, build = factory, build = builder, or build = constructor({written}) to call a function of that name"
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
        "leaf_build = {written} cannot build the container's fields that have no deps; write leaf_build = default, or leaf_build = constructor with the function's name in brackets, such as leaf_build = constructor(open)"
    )]
    InvalidLeafBuild { written: String },

    #[error(
        "field '{dependency}' is built with build = default, which takes no dependencies; remove its deps, or build it another way"
    )]
    DefaultWithDependencies { dependency: DependencyName },

    #[error("field '{dependency}' is built with {build}")]
    BuildCannotFail {
        dependency: DependencyName,
        build: InfallibleBuild,
    },

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

    #[error("{0}")]
    Async(AsyncProblem),

    #[error("field '{dependency}' {}", problem.worded(dependency))]
    TraitObject {
        dependency: DependencyName,
        problem: TraitObjectProblem,
    },

    #[error(
        "builder({written}) on field '{dependency}' cannot be read; builder takes start and finish, each with a function's name, such as builder(start = new_builder, finish = done)"
    )]
    InvalidBuilderSetting {
        dependency: DependencyName,
        written: String,
    },

    #[error(
        "the dependency '{label} = {field}' of field '{dependency}' has a label, which names a builder setter; write {field} alone, or build the field with build = builder"
    )]
    LabelWithoutBuilder {
        dependency: DependencyName,
        label: SetterName,
        field: DependencyName,
    },

    #[error(
        "field '{dependency}' lists '{prerequisite}' in deps more than once; list each dependency once"
    )]
    RepeatedPrerequisite {
        dependency: DependencyName,
        prerequisite: DependencyName,
    },

    #[error(
        "field '{dependency}' passes more than one dependency to the builder setter '{setter}'; give each dependency its own setter"
    )]
    DuplicateSetter {
        dependency: DependencyName,
        setter: SetterName,
    },

    #[error(
        "{feature} is unstable in this version of snapfit; enable the `unstable` feature of the `snapfit` crate to use it"
    )]
    RequiresUnstable { feature: Feature },

    #[error("{0}")]
    DispatchedTrait(DispatchProblem),

    #[error("{feature} is planned for a future version of snapfit and is not implemented yet")]
    NotYetImplemented { feature: Feature },
}
