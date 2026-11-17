//! Service: plans a container, reporting every problem its declaration has at once.

use crate::domain::{ConstructionPlan, ContainerDeclaration, DomainErrors, FeatureGate};
use crate::use_cases::declaration_rules::rule_violations;
use crate::use_cases::execution::container_execution;
use crate::use_cases::resolver::resolve;

#[cfg(test)]
mod tests;

pub(crate) fn plan_container<C, F>(
    container: ContainerDeclaration<C, F>,
    gate: FeatureGate,
) -> Result<ConstructionPlan<C, F>, DomainErrors> {
    let mut refusals = gate.refusals(&container);
    refusals.extend(rule_violations(&container));
    let checked = DomainErrors::check(refusals);
    let execution = container_execution(&container);
    let steps = resolve(container, gate);
    let ((_, execution), steps) =
        DomainErrors::both(DomainErrors::both(checked, execution), steps)?;
    Ok(steps.into_plan(execution))
}
