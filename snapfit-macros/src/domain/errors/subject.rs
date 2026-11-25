//! Error: which field or trait item an error is about, for the diagnostic to point at.

use super::{DomainError, Feature, FeatureProblem};
use crate::domain::dispatched_trait::ItemPosition;
use crate::domain::dispatched_type::AttributePosition;
use crate::domain::names::DependencyName;

impl DomainError {
    /// The dependency the error is about, whose field the diagnostic points at.
    pub(crate) fn subject(&self) -> Option<&DependencyName> {
        match self {
            Self::Declaration(problem) => problem.subject(),
            Self::Attribute(problem) => problem.subject(),
            Self::Graph(problem) => Some(problem.subject()),
            Self::Build(problem) => problem.subject(),
            Self::Builder(problem) => Some(problem.subject()),
            Self::Async(problem) => Some(problem.subject()),
            Self::TraitObject { dependency, .. } => Some(dependency),
            Self::StaticDispatch(problem) => problem.subject(),
            Self::Feature(problem) => problem.subject(),
            Self::DispatchedTrait(_) => None,
        }
    }
}

impl DomainError {
    /// The container's dispatch attribute the error is about, which the
    /// diagnostic points at instead of a field or the container's name.
    pub(crate) fn attribute_position(&self) -> Option<AttributePosition> {
        match self {
            Self::StaticDispatch(problem) => problem.position(),
            _ => None,
        }
    }
}

/// What of a dispatched trait an error is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DispatchSubject {
    Item(ItemPosition),
    /// The `unsafe` of an `unsafe trait`.
    Safety,
    /// What the attribute was given between its parentheses.
    Arguments,
    /// The item the attribute is on.
    Annotated,
}

impl DomainError {
    /// What of a dispatched trait the error is about; an error only a
    /// container can have is about the item as a whole.
    pub(crate) fn dispatch_subject(&self) -> DispatchSubject {
        match self {
            Self::DispatchedTrait(problem) => problem.subject(),
            Self::Feature(FeatureProblem::NotYetImplemented(Feature::RemoteStaticDispatch)) => {
                DispatchSubject::Arguments
            }
            _ => DispatchSubject::Annotated,
        }
    }
}
