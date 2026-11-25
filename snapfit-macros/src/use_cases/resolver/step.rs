//! Policy: the step that builds a declared dependency, which the resolver places.

use std::rc::Rc;

use crate::domain::{Call, DependencyDeclaration, Dispatch, Step};

/// The step that builds `declaration` by `call`, from the steps of its prerequisites.
pub(super) fn step_of<F, T>(
    declaration: DependencyDeclaration<F, T>,
    call: Call,
    dispatch: Dispatch,
    prerequisites: Vec<Rc<Step<F>>>,
) -> Step<F> {
    let DependencyDeclaration {
        name,
        syntax,
        lifecycle,
        execution,
        fallibility,
        ..
    } = declaration;
    let mut step = Step::new(
        name,
        syntax,
        lifecycle,
        execution,
        fallibility,
        call,
        prerequisites,
    );
    step.dispatch = dispatch;
    step
}
