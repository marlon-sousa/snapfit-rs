//! Policy: what a declaration may not combine, whichever build of snapfit runs.

use crate::domain::{ContainerDeclaration, DependencyDeclaration, DomainError, MacroForm};

#[cfg(test)]
mod tests;

/// Every rule a dependency breaks: a transient is read through a synchronous
/// getter, so it cannot be awaited, and `#[derive(Container)]` cannot leave a
/// transient or dispatched field out of the struct it does not rewrite.
pub(crate) fn rule_violations<C, F>(container: &ContainerDeclaration<C, F>) -> Vec<DomainError> {
    let form = container.form;
    container
        .dependencies
        .iter()
        .flat_map(|dependency| violations_of(form, dependency))
        .collect()
}

fn violations_of<F>(form: MacroForm, dependency: &DependencyDeclaration<F>) -> Vec<DomainError> {
    let name = &dependency.name;
    let is_async_transient = dependency.lifecycle.is_transient() && dependency.execution.is_async();
    let needs_struct_rewrite = dependency.lifecycle.is_transient() || dependency.dispatch.is_some();
    [
        is_async_transient.then(|| DomainError::AsyncTransient {
            dependency: name.clone(),
        }),
        (form.is_derive() && needs_struct_rewrite).then(|| DomainError::NeedsStructRewrite {
            dependency: name.clone(),
        }),
    ]
    .into_iter()
    .flatten()
    .collect()
}
