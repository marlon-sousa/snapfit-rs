//! Error: which field or trait item an error is about, for the diagnostic to point at.

use super::{DispatchProblem, DomainError, Feature};
use crate::domain::dispatched_trait::ItemPosition;
use crate::domain::names::DependencyName;

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
            | Self::BuildCannotFail { dependency, .. }
            | Self::QuotedFunctionName { dependency, .. }
            | Self::NotAFunctionName { dependency, .. }
            | Self::TraitObject { dependency, .. }
            | Self::InvalidBuilderSetting { dependency, .. }
            | Self::LabelWithoutBuilder { dependency, .. }
            | Self::RepeatedPrerequisite { dependency, .. }
            | Self::DuplicateSetter { dependency, .. } => Some(dependency),
            Self::Async(problem) => Some(problem.dependency()),
            Self::RequiresUnstable { feature } | Self::NotYetImplemented { feature } => {
                feature.dependency()
            }
            Self::BlankName { .. }
            | Self::TupleStruct
            | Self::MultipleContainerAttributes
            | Self::InvalidLeafBuild { .. }
            | Self::ContainerAttribute { .. }
            | Self::DispatchedTrait(_) => None,
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

impl DispatchProblem {
    fn subject(&self) -> DispatchSubject {
        match self {
            Self::Item { position, .. } => DispatchSubject::Item(*position),
            Self::UnsafeTrait(_) => DispatchSubject::Safety,
            Self::Arguments => DispatchSubject::Arguments,
            Self::NotATrait => DispatchSubject::Annotated,
        }
    }
}

impl DomainError {
    /// What of a dispatched trait the error is about; an error only a
    /// container can have is about the item as a whole.
    pub(crate) fn dispatch_subject(&self) -> DispatchSubject {
        match self {
            Self::DispatchedTrait(problem) => problem.subject(),
            Self::NotYetImplemented {
                feature: Feature::RemoteStaticDispatch,
            } => DispatchSubject::Arguments,
            _ => DispatchSubject::Annotated,
        }
    }
}
