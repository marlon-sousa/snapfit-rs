use super::*;
use crate::test_support::dispatched_trait;

#[test]
fn no_arguments_and_a_safe_trait_are_not_refused() {
    assert_eq!(arguments_refusal(DispatchArguments::None), None);
    assert_eq!(safety_refusal(&dispatched_trait(Vec::new())), None);
}

#[test]
fn remote_is_refused_as_planned_and_any_other_argument_as_unknown() {
    assert_eq!(
        arguments_refusal(DispatchArguments::Remote),
        Some(DomainError::Feature(FeatureProblem::NotYetImplemented(
            Feature::RemoteStaticDispatch
        )))
    );
    assert_eq!(
        arguments_refusal(DispatchArguments::Other),
        Some(DomainError::DispatchedTrait(DispatchProblem::Arguments))
    );
}

#[test]
fn an_unsafe_trait_is_refused() {
    let mut declaration = dispatched_trait(Vec::new());
    declaration.safety = Safety::Unsafe;

    assert_eq!(
        safety_refusal(&declaration),
        Some(DomainError::DispatchedTrait(DispatchProblem::UnsafeTrait(
            declaration.name.clone()
        )))
    );
}
