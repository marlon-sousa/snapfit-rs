//! Policy: the resolved steps in both orders, waiting for how the container is executed.

use std::rc::Rc;

use super::walk::Placed;
use crate::domain::{ConstructionPlan, DispatchedTypePlan, Execution, Fallibility, Step};

#[derive(Debug)]
pub(crate) struct ResolvedSteps<C, F> {
    syntax: C,
    construction_order: Vec<Rc<Step<F>>>,
    declaration_order: Vec<Rc<Step<F>>>,
}

impl<C, F> ResolvedSteps<C, F> {
    pub(super) fn new(syntax: C, construction_order: Vec<Placed<F>>) -> Self {
        let mut declaration_order: Vec<_> =
            construction_order.iter().map(cloned_placement).collect();
        declaration_order.sort_by_key(position);
        Self {
            syntax,
            construction_order: construction_order.into_iter().map(step).collect(),
            declaration_order: declaration_order.into_iter().map(step).collect(),
        }
    }

    pub(crate) fn into_plan<S, T>(
        self,
        execution: Execution,
        fallibility: Fallibility,
        dispatched_types: Vec<DispatchedTypePlan<S, T>>,
    ) -> ConstructionPlan<C, F, S, T> {
        ConstructionPlan::new(
            self.syntax,
            execution,
            fallibility,
            self.construction_order,
            self.declaration_order,
            dispatched_types,
        )
    }
}

fn cloned_placement<F>(placed: &Placed<F>) -> Placed<F> {
    Placed {
        position: placed.position,
        step: Rc::clone(&placed.step),
    }
}

fn position<F>(placed: &Placed<F>) -> usize {
    placed.position
}

fn step<F>(placed: Placed<F>) -> Rc<Step<F>> {
    placed.step
}
