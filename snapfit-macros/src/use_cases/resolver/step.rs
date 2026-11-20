//! Policy: the step that builds a declared dependency, which the resolver places.

use std::rc::Rc;

use crate::domain::{Call, DependencyDeclaration, Step};

/// The step that builds `declaration` by `call`, from the steps of its prerequisites.
pub(super) fn step_of<F>(
    declaration: DependencyDeclaration<F>,
    call: Call,
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
    Step::new(
        name,
        syntax,
        lifecycle,
        execution,
        fallibility,
        call,
        prerequisites,
    )
}
