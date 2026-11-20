//! Test support: the names and declarations that the tests of several modules build.

use crate::domain::{
    BuilderMethods, Construction, ContainerDeclaration, ContainerName, DependencyDeclaration,
    DependencyName, DomainError, FunctionName, Prerequisite, SetterName,
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

/// `field` passed to the builder setter `label`, as `deps = [label = field]` says.
pub(crate) fn labelled(field: &str, label: &str) -> Prerequisite {
    Prerequisite {
        field: name(field),
        label: Some(SetterName::new(label).unwrap()),
    }
}

/// The functions of `build = builder` with no settings.
pub(crate) fn builder_methods() -> BuilderMethods {
    BuilderMethods::new(
        FunctionName::new("builder").unwrap(),
        FunctionName::new("build").unwrap(),
    )
}

/// `build = builder` with no settings.
pub(crate) fn builder() -> Construction {
    Construction::Builder(builder_methods())
}

/// Siblings written alone in `deps`, without labels.
pub(crate) fn prerequisites(texts: &[&str]) -> Vec<Prerequisite> {
    names(texts).into_iter().map(Prerequisite::new).collect()
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
    dependency.prerequisites = self::prerequisites(prerequisites);
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

/// What a `macro_rules!` passes for `$e:expr`: the expression in an invisible group.
pub(crate) fn through_a_macro(tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    proc_macro2::TokenTree::Group(proc_macro2::Group::new(
        proc_macro2::Delimiter::None,
        tokens,
    ))
    .into()
}

/// Tokens as text without whitespace, so that a test compares code, not layout.
pub(crate) fn compact(tokens: impl ToString) -> String {
    tokens.to_string().split_whitespace().collect()
}
