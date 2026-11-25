//! Policy: whether the container is built at once by `new()` or awaited through `init()`.

use crate::domain::{
    AsyncProblem, ContainerDeclaration, DependencyDeclaration, DomainError, DomainErrors, Execution,
};

#[cfg(test)]
mod tests;

/// The declared mode, or async when any dependency is. A synchronous container
/// cannot await, so every async dependency it holds is reported.
pub(crate) fn container_execution<C, F, S, T>(
    container: &ContainerDeclaration<C, F, S, T>,
) -> Result<Execution, DomainErrors> {
    let mut async_dependencies = container.dependencies.iter().filter(is_async).peekable();
    match container.mode {
        Some(Execution::Async) => Ok(Execution::Async),
        Some(Execution::Sync) => {
            DomainErrors::check(async_dependencies.map(async_in_sync_container).collect())?;
            Ok(Execution::Sync)
        }
        None if async_dependencies.peek().is_some() => Ok(Execution::Async),
        None => Ok(Execution::Sync),
    }
}

fn is_async<F, T>(dependency: &&DependencyDeclaration<F, T>) -> bool {
    dependency.execution.is_async()
}

fn async_in_sync_container<F, T>(dependency: &DependencyDeclaration<F, T>) -> DomainError {
    AsyncProblem::InSyncContainer(dependency.name.clone()).into()
}
