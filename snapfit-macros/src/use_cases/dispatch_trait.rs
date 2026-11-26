//! Service: plans the impls of a dispatched trait, reporting every problem
//! of the attribute and of the trait at once.

use crate::domain::{
    Annotated, DispatchAttribute, DispatchPlan, DispatchProblem, DomainErrors, TraitDeclaration,
};
use crate::use_cases::dispatch_rules::{arguments_refusals, bounds_refusal, safety_refusal};
use crate::use_cases::forwarding::forwarding;

#[cfg(test)]
mod tests;

pub(crate) fn plan_dispatch<T, M, A>(
    attribute: DispatchAttribute<T, M, A>,
) -> Result<DispatchPlan<T, M, A>, DomainErrors> {
    let arguments = arguments_refusals(attribute.arguments);
    match attribute.annotated {
        Annotated::Trait(declaration) => {
            let header = DomainErrors::check(
                arguments
                    .into_iter()
                    .chain(safety_refusal(&declaration))
                    .chain(bounds_refusal(&declaration))
                    .collect(),
            );
            let (_, plan) = DomainErrors::both(header, plan_items(declaration))?;
            Ok(plan)
        }
        // The trait's own problems are the last mark's to report, once.
        Annotated::MarkedAgain(name) => Err(DomainErrors::new(
            DispatchProblem::MarkedAgain(name).into(),
            arguments,
        )),
        Annotated::Other => Err(DomainErrors::new(
            DispatchProblem::NotATrait.into(),
            arguments,
        )),
    }
}

fn plan_items<T, M, A>(
    declaration: TraitDeclaration<T, M, A>,
) -> Result<DispatchPlan<T, M, A>, DomainErrors> {
    let name = declaration.name;
    let items = DomainErrors::collect(
        declaration
            .items
            .into_iter()
            .map(|item| forwarding(&name, item)),
    )?;
    Ok(DispatchPlan::new(declaration.syntax, items))
}
