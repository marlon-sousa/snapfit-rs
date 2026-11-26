//! Policy: what the attribute and the trait as a whole must be for snapfit to
//! dispatch the trait statically.

use crate::domain::{
    ArgumentProblem, DispatchArguments, DispatchProblem, DispatchSetting, DomainError, Feature,
    FeatureProblem, RuntimeValue, Safety, SelfUse, SettingPosition, SettingProblem,
    TraitDeclaration,
};

#[cfg(test)]
mod tests;

/// The refusal of each setting the attribute was given, at that setting:
/// `remote` is planned, `crate` takes a path, once, and nothing else is
/// taken.
pub(crate) fn arguments_refusals(arguments: DispatchArguments) -> Vec<DomainError> {
    arguments
        .settings
        .into_iter()
        .scan(CrateSeen::No, with_repetition)
        .enumerate()
        .flat_map(setting_refusals)
        .collect()
}

/// Whether a `crate` was read among the settings before.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrateSeen {
    No,
    Yes,
}

/// Whether a setting is a `crate` written after another one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Repetition {
    Not,
    Repeated,
}

/// `setting` with whether it repeats a `crate` read before it.
fn with_repetition(
    seen: &mut CrateSeen,
    setting: DispatchSetting,
) -> Option<(DispatchSetting, Repetition)> {
    let repetition = match (&setting, *seen) {
        (DispatchSetting::Runtime(_), CrateSeen::Yes) => Repetition::Repeated,
        _ => Repetition::Not,
    };
    if setting.is_runtime() {
        *seen = CrateSeen::Yes;
    }
    Some((setting, repetition))
}

fn setting_refusals(
    (index, (setting, repetition)): (usize, (DispatchSetting, Repetition)),
) -> Vec<DomainError> {
    let position = SettingPosition::new(index);
    setting_problems(setting, repetition)
        .into_iter()
        .map(|problem| setting_error(position, problem))
        .collect()
}

/// What is wrong with one setting. A `crate` after another one is written
/// more than once, whatever else is wrong with it.
fn setting_problems(setting: DispatchSetting, repetition: Repetition) -> Vec<SettingProblem> {
    match setting {
        DispatchSetting::Remote => vec![SettingProblem::Planned(
            FeatureProblem::NotYetImplemented(Feature::RemoteStaticDispatch),
        )],
        DispatchSetting::Other => vec![SettingProblem::Unknown],
        DispatchSetting::Runtime(value) => runtime_problem(value)
            .into_iter()
            .chain((repetition == Repetition::Repeated).then(repeated_runtime_path))
            .map(SettingProblem::Runtime)
            .collect(),
    }
}

/// The problem of what `crate = ...` was given, in the container's words.
fn runtime_problem(value: RuntimeValue) -> Option<ArgumentProblem> {
    match value {
        RuntimeValue::Path => None,
        RuntimeValue::Quoted(text) => Some(ArgumentProblem::QuotedRuntimePath(text)),
        RuntimeValue::NotAPath(text) => Some(ArgumentProblem::NotARuntimePath(text)),
        RuntimeValue::Bare => Some(ArgumentProblem::BareRuntimePath),
    }
}

/// A second `crate`, in the words `#[container]` gives it.
fn repeated_runtime_path() -> ArgumentProblem {
    ArgumentProblem::RepeatedSetting("crate".to_owned())
}

fn setting_error(position: SettingPosition, problem: SettingProblem) -> DomainError {
    DispatchProblem::Setting { position, problem }.into()
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
