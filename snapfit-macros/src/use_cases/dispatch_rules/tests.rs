use super::*;
use crate::test_support::dispatched_trait;

fn at(index: usize, problem: SettingProblem) -> DomainError {
    setting_error(SettingPosition::new(index), problem)
}

fn repeated_crate() -> SettingProblem {
    SettingProblem::Runtime(ArgumentProblem::RepeatedSetting("crate".to_owned()))
}

fn planned_remote() -> SettingProblem {
    SettingProblem::Planned(FeatureProblem::NotYetImplemented(
        Feature::RemoteStaticDispatch,
    ))
}

#[test]
fn no_arguments_and_a_safe_trait_are_not_refused() {
    assert_eq!(arguments_refusals(DispatchArguments::default()), Vec::new());
    assert_eq!(safety_refusal(&dispatched_trait(Vec::new())), None);
}

#[test]
fn one_crate_alone_is_taken() {
    let given = DispatchArguments::new(vec![DispatchSetting::Runtime(RuntimeValue::Path)]);
    assert_eq!(arguments_refusals(given), Vec::new());
}

#[test]
fn every_remote_is_refused_as_planned_at_its_own_setting() {
    let given = DispatchArguments::new(vec![DispatchSetting::Remote, DispatchSetting::Remote]);
    assert_eq!(
        arguments_refusals(given),
        vec![at(0, planned_remote()), at(1, planned_remote())]
    );
}

#[test]
fn a_planned_remote_is_refused_beside_every_other_mistake() {
    let given = DispatchArguments::new(vec![
        DispatchSetting::Runtime(RuntimeValue::Quoted("snapfit".to_owned())),
        DispatchSetting::Runtime(RuntimeValue::NotAPath("3".to_owned())),
        DispatchSetting::Remote,
        DispatchSetting::Other,
    ]);
    assert_eq!(
        arguments_refusals(given),
        vec![
            at(
                0,
                SettingProblem::Runtime(ArgumentProblem::QuotedRuntimePath("snapfit".to_owned()))
            ),
            at(
                1,
                SettingProblem::Runtime(ArgumentProblem::NotARuntimePath("3".to_owned()))
            ),
            at(1, repeated_crate()),
            at(2, planned_remote()),
            at(3, SettingProblem::Unknown),
        ]
    );
}

#[test]
fn every_crate_after_the_first_is_refused_as_written_again() {
    let path = || DispatchSetting::Runtime(RuntimeValue::Path);
    let given = DispatchArguments::new(vec![path(), DispatchSetting::Other, path(), path()]);
    assert_eq!(
        arguments_refusals(given),
        vec![
            at(1, SettingProblem::Unknown),
            at(2, repeated_crate()),
            at(3, repeated_crate()),
        ]
    );
}

#[test]
fn a_crate_with_no_value_is_refused_as_needing_one() {
    let given = DispatchArguments::new(vec![DispatchSetting::Runtime(RuntimeValue::Bare)]);
    assert_eq!(
        arguments_refusals(given),
        vec![at(
            0,
            SettingProblem::Runtime(ArgumentProblem::BareRuntimePath)
        )]
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
