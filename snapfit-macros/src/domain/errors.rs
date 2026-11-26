//! Error: what makes snapfit refuse a container or a dispatched trait, each
//! worded as the sentence the compiler shows, which must read well spoken by
//! a screen reader.

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
pub(crate) use dispatched_trait::{DispatchProblem, ItemProblem, SettingProblem};
pub(crate) use feature::{Feature, FeatureProblem};
pub(crate) use graph::GraphProblem;
pub(crate) use static_dispatch::{DispatchAttributeKind, StaticDispatchProblem, Unreadability};
pub(crate) use subject::DispatchSubject;
pub(crate) use trait_objects::TraitObjectProblem;

#[cfg(test)]
mod tests;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomainError {
    #[error(transparent)]
    Declaration(#[from] DeclarationProblem),
    #[error(transparent)]
    Attribute(#[from] AttributeProblem),
    #[error(transparent)]
    Graph(#[from] GraphProblem),
    #[error(transparent)]
    Build(#[from] BuildProblem),
    #[error(transparent)]
    Builder(#[from] BuilderProblem),
    #[error(transparent)]
    Async(#[from] AsyncProblem),
    #[error("field '{dependency}' {}", problem.worded(dependency))]
    TraitObject {
        dependency: DependencyName,
        problem: TraitObjectProblem,
    },
    #[error(transparent)]
    DispatchedTrait(#[from] DispatchProblem),
    #[error(transparent)]
    StaticDispatch(#[from] StaticDispatchProblem),
    #[error(transparent)]
    Feature(#[from] FeatureProblem),
}
