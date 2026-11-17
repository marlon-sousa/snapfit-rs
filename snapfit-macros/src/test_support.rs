//! Test support: the names and declarations that the tests of several modules build.

use crate::domain::{
    ContainerDeclaration, ContainerName, DependencyDeclaration, DependencyName, DomainError,
};

pub(crate) fn name(text: &str) -> DependencyName {
    DependencyName::new(text).unwrap()
}

/// A cycle from `start` through `through`, back to `start`.
pub(crate) fn cycle(start: &str, through: &[&str]) -> DomainError {
    DomainError::DependencyCycle {
        start: name(start),
        through: names(through),
    }
}

pub(crate) fn names(texts: &[&str]) -> Vec<DependencyName> {
    texts.iter().copied().map(name).collect()
}

/// A singleton built with `new`, needing `prerequisites`.
pub(crate) fn dependency(text: &str, prerequisites: &[&str]) -> DependencyDeclaration<()> {
    dependency_with(text, prerequisites, ())
}

pub(crate) fn dependency_with<F>(
    text: &str,
    prerequisites: &[&str],
    syntax: F,
) -> DependencyDeclaration<F> {
    let mut dependency = DependencyDeclaration::new(name(text), syntax);
    dependency.prerequisites = names(prerequisites);
    dependency
}

/// A `#[container]` named `App` holding `dependencies`.
pub(crate) fn container<F>(
    dependencies: Vec<DependencyDeclaration<F>>,
) -> ContainerDeclaration<(), F> {
    let mut container = ContainerDeclaration::new(ContainerName::new("App").unwrap(), ());
    container.dependencies = dependencies;
    container
}

/// Tokens as text without whitespace, so that a test compares code, not layout.
pub(crate) fn compact(tokens: impl ToString) -> String {
    tokens.to_string().split_whitespace().collect()
}
