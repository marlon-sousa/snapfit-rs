//! Error: why a container cannot dispatch a type statically as its
//! `static_dispatch_for` and `extend_static_dispatch` say, or a field cannot
//! hold that type.

use derive_more::Display;

use crate::domain::dispatched_type::AttributePosition;
use crate::domain::names::{DependencyName, TypeName};

mod unreadability;
mod wording;

pub(crate) use unreadability::Unreadability;
use wording::{example, undeclared};

/// Which of the container's dispatch attributes an error is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub(crate) enum DispatchAttributeKind {
    #[display("static_dispatch_for")]
    Declaration,
    #[display("extend_static_dispatch")]
    Extension,
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum StaticDispatchProblem {
    #[display(
        "{kind} cannot be read: {problem}; write it as {}",
        example(*kind)
    )]
    Unreadable {
        position: AttributePosition,
        kind: DispatchAttributeKind,
        problem: Unreadability,
    },
    #[display(
        "static_dispatch_for needs the name the fields write, after the trait and the word as, such as static_dispatch_for(Database as Db)"
    )]
    WithoutName { position: AttributePosition },
    /// The name as written, such as `shop::Db`.
    #[display(
        "{kind} names the dispatched type through the path {written}; write a single name, such as {}",
        example(*kind)
    )]
    NameAsPath {
        position: AttributePosition,
        kind: DispatchAttributeKind,
        written: String,
    },
    /// The setting as written, up to its value.
    #[display(
        "static_dispatch_for takes the trait and its name and nothing else, not {written}; remove {written}, and add implementations with implementation on a field or with extend_static_dispatch, such as extend_static_dispatch(Db, [Sqlite])"
    )]
    UnknownSetting {
        position: AttributePosition,
        written: String,
    },
    /// The value as written.
    #[display(
        "{kind} takes its implementations in square brackets, not {written}; write them as {}",
        example(*kind)
    )]
    NotAList {
        position: AttributePosition,
        kind: DispatchAttributeKind,
        written: String,
    },
    /// The entry as written, such as `impl Store` or `_`.
    #[display(
        "{written} in {kind} is not a type the container can name; write each implementation in full, such as {}",
        example(*kind)
    )]
    NotAType {
        position: AttributePosition,
        kind: DispatchAttributeKind,
        written: String,
    },
    #[display(
        "extend_static_dispatch({name}) needs the implementations it adds, in square brackets after the name, such as extend_static_dispatch({name}, [Mock])"
    )]
    WithoutList {
        position: AttributePosition,
        name: TypeName,
    },
    #[display(
        "the dispatched type '{name}' is declared more than once; give each static_dispatch_for a name of its own"
    )]
    RepeatedName {
        position: AttributePosition,
        name: TypeName,
    },
    #[display("{}", undeclared(name, declared))]
    Undeclared {
        position: AttributePosition,
        name: TypeName,
        declared: Vec<TypeName>,
    },
    #[display(
        "the dispatched type '{name}' holds no implementation in this build; name one with implementation on a field of type Arc<{name}>, list those a factory returns with implementations = [...] beside build = factory, or add one with extend_static_dispatch({name}, [...])"
    )]
    WithoutImplementations {
        position: AttributePosition,
        name: TypeName,
    },
    #[display(
        "field '{dependency}' holds the dispatched type {name}, whose implementations the container cannot choose between; name the one it builds, such as implementation = MySql, or add build = factory to the field and build it in the build_{} method of the container's factories trait",
        dependency.without_raw_prefix()
    )]
    WithoutImplementation {
        dependency: DependencyName,
        name: TypeName,
    },
    #[display(
        "field '{dependency}' lists implementations of the dispatched type {name}, which only a factory can choose between; add build = factory to the field and return one of them from the build_{} method of the container's factories trait, or name the one it builds with implementation, such as implementation = MySql",
        dependency.without_raw_prefix()
    )]
    ListWithoutFactory {
        dependency: DependencyName,
        name: TypeName,
    },
    /// The implementation as written, such as `MySql`.
    #[display(
        "field '{dependency}' names the one implementation {implementation} of the dispatched type {name}, and is built with build = factory, which returns the field itself; list the variants its factory may return with implementations = [{implementation}], or remove build = factory so that the container builds {implementation}"
    )]
    ImplementationWithFactory {
        dependency: DependencyName,
        name: TypeName,
        implementation: TypeName,
    },
    /// The path as written, such as `crate::shop::Db`.
    #[display(
        "field '{dependency}' names the dispatched type {name} through the path {written}; write {name} as static_dispatch_for declares it"
    )]
    ThroughPath {
        dependency: DependencyName,
        name: TypeName,
        written: String,
    },
    #[display(
        "field '{dependency}' is marked #[dispatch(...)], which snapfit does not support; declare the dispatched type under #[container] with static_dispatch_for, such as #[static_dispatch_for(Store as Stores)], and give the field that type with its implementation, such as Arc<Stores>"
    )]
    OldAttribute { dependency: DependencyName },
}

impl StaticDispatchProblem {
    pub(crate) fn subject(&self) -> Option<&DependencyName> {
        match self {
            Self::WithoutImplementation { dependency, .. }
            | Self::ListWithoutFactory { dependency, .. }
            | Self::ImplementationWithFactory { dependency, .. }
            | Self::ThroughPath { dependency, .. }
            | Self::OldAttribute { dependency } => Some(dependency),
            Self::Unreadable { .. }
            | Self::WithoutName { .. }
            | Self::NameAsPath { .. }
            | Self::UnknownSetting { .. }
            | Self::NotAList { .. }
            | Self::NotAType { .. }
            | Self::WithoutList { .. }
            | Self::RepeatedName { .. }
            | Self::Undeclared { .. }
            | Self::WithoutImplementations { .. } => None,
        }
    }

    pub(crate) fn position(&self) -> Option<AttributePosition> {
        match self {
            Self::Unreadable { position, .. }
            | Self::WithoutName { position }
            | Self::NameAsPath { position, .. }
            | Self::UnknownSetting { position, .. }
            | Self::NotAList { position, .. }
            | Self::NotAType { position, .. }
            | Self::WithoutList { position, .. }
            | Self::RepeatedName { position, .. }
            | Self::Undeclared { position, .. }
            | Self::WithoutImplementations { position, .. } => Some(*position),
            Self::WithoutImplementation { .. }
            | Self::ListWithoutFactory { .. }
            | Self::ImplementationWithFactory { .. }
            | Self::ThroughPath { .. }
            | Self::OldAttribute { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests;
