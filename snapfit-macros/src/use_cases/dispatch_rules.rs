//! Policy: what the attribute and the trait as a whole must be for snapfit to
//! dispatch the trait statically.

use crate::domain::{
    DispatchArguments, DispatchProblem, DomainError, Feature, FeatureProblem, Safety, SelfUse,
    TraitDeclaration,
};

#[cfg(test)]
mod tests;

/// The refusal of what the attribute was given, if any.
pub(crate) fn arguments_refusal(arguments: DispatchArguments) -> Option<DomainError> {
    match arguments {
        DispatchArguments::None => None,
        DispatchArguments::Remote => {
            Some(FeatureProblem::NotYetImplemented(Feature::RemoteStaticDispatch).into())
        }
        DispatchArguments::Other => Some(DispatchProblem::Arguments.into()),
    }
}

/// The refusal of the trait's own header, if any.
pub(crate) fn safety_refusal<T, M, A>(
    declaration: &TraitDeclaration<T, M, A>,
) -> Option<DomainError> {
    match declaration.safety {
        Safety::Safe => None,
        Safety::Unsafe => Some(DispatchProblem::UnsafeTrait(declaration.name.clone()).into()),
    }
}

/// The refusal of the trait's own bounds, if they name `Self` where an
/// implementation's type could not stand for it.
pub(crate) fn bounds_refusal<T, M, A>(
    declaration: &TraitDeclaration<T, M, A>,
) -> Option<DomainError> {
    match declaration.bounds {
        SelfUse::Forwardable => None,
        SelfUse::Elsewhere => {
            Some(DispatchProblem::SelfInTraitBounds(declaration.name.clone()).into())
        }
    }
}
