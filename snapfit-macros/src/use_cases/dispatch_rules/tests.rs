use super::*;
use crate::test_support::dispatched_trait;

#[test]
fn no_arguments_and_a_safe_trait_are_not_refused() {
    assert_eq!(arguments_refusal(DispatchArguments::default()), None);
    assert_eq!(safety_refusal(&dispatched_trait(Vec::new())), None);
}

#[test]
fn one_crate_alone_is_taken() {
    assert_eq!(arguments_refusal(DispatchArguments::new(0, 1, 0)), None);
}

#[test]
fn remote_is_refused_as_planned_whatever_else_is_given() {
    let planned = Some(DomainError::Feature(FeatureProblem::NotYetImplemented(
        Feature::RemoteStaticDispatch,
    )));
    let given = [
        DispatchArguments::new(1, 0, 0),
        DispatchArguments::new(1, 2, 1),
        DispatchArguments::new(2, 0, 0),
    ];
    for arguments in given {
        assert_eq!(arguments_refusal(arguments), planned, "{arguments:?}");
    }
}

#[test]
fn another_argument_or_a_second_crate_is_refused_as_unknown() {
    let unknown = Some(DomainError::DispatchedTrait(DispatchProblem::Arguments));
    let given = [
        DispatchArguments::new(0, 0, 1),
        DispatchArguments::new(0, 2, 0),
        DispatchArguments::new(0, 1, 1),
    ];
    for arguments in given {
        assert_eq!(arguments_refusal(arguments), unknown, "{arguments:?}");
    }
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

#[test]
fn a_trait_whose_bounds_name_self_is_refused() {
    let mut declaration = dispatched_trait(Vec::new());
    assert_eq!(bounds_refusal(&declaration), None);
    declaration.bounds = SelfUse::Elsewhere;

    assert_eq!(
        bounds_refusal(&declaration),
        Some(DomainError::DispatchedTrait(
            DispatchProblem::SelfInTraitBounds(declaration.name.clone())
        ))
    );
}
