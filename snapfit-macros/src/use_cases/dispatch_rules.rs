//! Policy: what the attribute and the trait as a whole must be for snapfit to
//! dispatch the trait statically.

use crate::domain::{
    DispatchArguments, DispatchProblem, DomainError, Feature, FeatureProblem, Safety,
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
