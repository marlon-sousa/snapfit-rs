//! Error: what makes snapfit refuse a container or a dispatched trait, each
//! worded as the sentence the compiler shows, which must read well spoken by
//! a screen reader. The error is composed of parts, one for each kind of
//! problem, each with its own words and subject.

mod accumulated;
mod arguments;
mod asynchrony;
mod attribute;
mod build;
mod builder;
mod declaration;
mod dispatched_trait;
mod feature;
mod graph;
mod sentences;
mod static_dispatch;
mod subject;
mod trait_objects;

use thiserror::Error;

use crate::domain::names::DependencyName;

pub(crate) use accumulated::DomainErrors;
pub(crate) use arguments::ArgumentProblem;
pub(crate) use asynchrony::AsyncProblem;
pub(crate) use attribute::AttributeProblem;
pub(crate) use build::{BuildProblem, InfallibleBuild};
pub(crate) use builder::BuilderProblem;
pub(crate) use declaration::DeclarationProblem;
pub(crate) use dispatched_trait::{DispatchProblem, ItemProblem};
pub(crate) use feature::{Feature, FeatureProblem};
pub(crate) use graph::GraphProblem;
pub(crate) use static_dispatch::{DispatchAttributeKind, StaticDispatchProblem, Unreadability};
pub(crate) use subject::DispatchSubject;
pub(crate) use trait_objects::TraitObjectProblem;

#[cfg(test)]
mod tests;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomainError {
    #[error("{0}")]
    Declaration(DeclarationProblem),
    #[error("{0}")]
    Attribute(AttributeProblem),
    #[error("{0}")]
    Graph(GraphProblem),
    #[error("{0}")]
    Build(BuildProblem),
    #[error("{0}")]
    Builder(BuilderProblem),
    #[error("{0}")]
    Async(AsyncProblem),
    #[error("field '{dependency}' {}", problem.worded(dependency))]
    TraitObject {
        dependency: DependencyName,
        problem: TraitObjectProblem,
    },
    #[error("{0}")]
    DispatchedTrait(DispatchProblem),
    #[error("{0}")]
    StaticDispatch(StaticDispatchProblem),
    #[error("{0}")]
    Feature(FeatureProblem),
}
