//! Service: plans a container, reporting every problem its declaration has at once.

use crate::domain::{
    ConstructionPlan, ContainerDeclaration, DispatchedTypePlan, DomainErrors, FeatureGate,
};
use crate::use_cases::declaration_rules::rule_violations;
use crate::use_cases::execution::container_execution;
use crate::use_cases::fallibility::container_fallibility;
use crate::use_cases::resolver::resolve;
use crate::use_cases::static_dispatch::{dispatch_violations, dispatched_types};

#[cfg(test)]
mod tests;

pub(crate) fn plan_container<C, F, S: Clone, T: Clone>(
    container: ContainerDeclaration<C, F, S, T>,
    gate: FeatureGate,
) -> Result<ConstructionPlan<C, F, S, T>, DomainErrors> {
    let mut refusals = gate.refusals(&container);
    refusals.extend(rule_violations(&container));
    refusals.extend(dispatch_violations(&container));
    let types = dispatched_types(&container);
    let checked = DomainErrors::check(refusals);
    let execution = container_execution(&container);
    let fallibility = container_fallibility(&container);
    let steps = resolve(container);
    let ((_, execution), steps) =
        DomainErrors::both(DomainErrors::both(checked, execution), steps)?;
    Ok(steps.into_plan(execution, fallibility, types))
}

/// The types a container that cannot be planned still declares, each with
/// the implementations it holds, so that its struct, kept as written, names
/// them.
pub(crate) fn declared_dispatched_types<C, F, S: Clone, T: Clone>(
    container: &ContainerDeclaration<C, F, S, T>,
) -> Vec<DispatchedTypePlan<S, T>> {
    dispatched_types(container)
}
