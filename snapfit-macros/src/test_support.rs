//! Test support: the names and declarations that the tests of several modules build.

use crate::domain::{
    Annotated, BuilderMethods, Construction, ContainerDeclaration, ContainerName,
    DependencyDeclaration, DependencyName, DispatchArguments, DispatchAttribute, DomainError,
    Execution, FunctionName, ItemPosition, MethodName, MethodShape, PlacedItem, Prerequisite,
    Receiver, ReturnShape, Safety, SelfBound, SelfUse, SetterName, TraitDeclaration,
    TraitItemDeclaration, TraitName,
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

/// A method named `name` of the shape given, synchronous and safe, bounding
/// `Self` by nothing, whose syntax is its name.
pub(crate) fn method(
    name: &str,
    receiver: Receiver,
    self_use: SelfUse,
    returns: ReturnShape,
) -> TraitItemDeclaration<&str, &str> {
    TraitItemDeclaration::Method {
        name: MethodName::new(name).unwrap(),
        syntax: name,
        shape: MethodShape::new(
            receiver,
            self_use,
            SelfBound::Forwardable,
            returns,
            Execution::Sync,
            Safety::Safe,
        ),
    }
}

/// A method taking `&self` that returns a plain type, which forwards as written.
pub(crate) fn forwardable_method(name: &str) -> TraitItemDeclaration<&str, &str> {
    method(
        name,
        Receiver::Reference,
        SelfUse::Forwardable,
        ReturnShape::Plain,
    )
}

/// `declaration` as the first item of its trait.
pub(crate) fn first<'a>(
    declaration: TraitItemDeclaration<&'a str, &'a str>,
) -> PlacedItem<&'a str, &'a str> {
    PlacedItem::new(ItemPosition::new(0), declaration)
}

/// `#[snapfit::static_dispatch]` with no arguments on a trait named
/// `Database` holding `items`.
pub(crate) fn dispatch_attribute<'a>(
    items: Vec<TraitItemDeclaration<&'a str, &'a str>>,
) -> DispatchAttribute<(), &'a str, &'a str> {
    DispatchAttribute::new(
        DispatchArguments::None,
        Annotated::Trait(dispatched_trait(items)),
    )
}

/// A safe trait named `Database` holding `items`, in order.
pub(crate) fn dispatched_trait<'a>(
    items: Vec<TraitItemDeclaration<&'a str, &'a str>>,
) -> TraitDeclaration<(), &'a str, &'a str> {
    let items = items.into_iter().enumerate().map(placed).collect();
    TraitDeclaration::new(TraitName::new("Database").unwrap(), (), Safety::Safe, items)
}

fn placed<'a>(
    (position, declaration): (usize, TraitItemDeclaration<&'a str, &'a str>),
) -> PlacedItem<&'a str, &'a str> {
    PlacedItem::new(ItemPosition::new(position), declaration)
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
