//! Error: which field an error is about, for the diagnostic to point at.

use super::DomainError;
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
            | Self::AsyncDependencyInSyncContainer { dependency }
            | Self::AsyncTransient { dependency }
            | Self::AsyncDefault { dependency }
            | Self::TraitObjectWithoutFactory { dependency }
            | Self::InvalidBuilderSetting { dependency, .. }
            | Self::LabelWithoutBuilder { dependency, .. }
            | Self::RepeatedPrerequisite { dependency, .. }
            | Self::DuplicateSetter { dependency, .. } => Some(dependency),
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
