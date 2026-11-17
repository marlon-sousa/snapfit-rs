//! Entity: a container as the user declared it, before snapfit checks or plans it.
//!
//! `C` and `F` are the syntax of the container and of each field, which the
//! domain carries without reading so that the emitter gets them back with
//! each decision.

use derive_new::new;
use strum::EnumIs;

use crate::domain::names::{ContainerName, DependencyName, FunctionName, TypeName};

#[cfg(test)]
mod tests;

#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Lifecycle {
    /// Built once when the container is, and shared by every reader.
    #[default]
    Singleton,
    /// Built again for every reader, and never stored.
    Transient,
}

#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Execution {
    #[default]
    Sync,
    /// Awaited, and its error returned from the container's `init`.
    Async,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Construction {
    /// An associated function of the dependency's type.
    Constructor(Constructor),
    /// An associated function of the container.
    Builder(FunctionName),
}

impl Default for Construction {
    fn default() -> Self {
        Self::Constructor(Constructor::New)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Constructor {
    New,
    Named(FunctionName),
}

/// What the field's type is, as far as building it goes.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TypeShape {
    #[default]
    Value,
    /// `Arc<T>`, whose `T` is the type the constructor belongs to.
    Arc,
    /// A type holding `dyn Trait`, which has no constructor of its own.
    TraitObject,
}

/// Which of snapfit's macros declared the container.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum MacroForm {
    /// `#[container]`, which rewrites the struct.
    #[default]
    Attribute,
    /// `#[derive(Container)]`, which can only add impls to the struct as written.
    Derive,
}

/// Whether the container struct declares type or lifetime parameters.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TypeParameters {
    #[default]
    None,
    Declared,
}

#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Introspection {
    #[default]
    Off,
    On,
}

/// `#[dispatch(...)]`: the implementations an enum would dispatch to.
#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnumDispatch {
    pub(crate) trait_name: TypeName,
    pub(crate) implementations: Vec<TypeName>,
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct DependencyDeclaration<F> {
    pub(crate) name: DependencyName,
    pub(crate) syntax: F,
    #[new(default)]
    pub(crate) lifecycle: Lifecycle,
    #[new(default)]
    pub(crate) execution: Execution,
    #[new(default)]
    pub(crate) construction: Construction,
    #[new(default)]
    pub(crate) shape: TypeShape,
    /// Siblings passed to the constructor or builder, in order.
    #[new(default)]
    pub(crate) prerequisites: Vec<DependencyName>,
    /// `parent(name)` entries of `deps`, which name a parent container's dependency.
    #[new(default)]
    pub(crate) parent_prerequisites: Vec<DependencyName>,
    #[new(default)]
    pub(crate) dispatch: Option<EnumDispatch>,
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContainerDeclaration<C, F> {
    pub(crate) name: ContainerName,
    pub(crate) syntax: C,
    #[new(default)]
    pub(crate) form: MacroForm,
    #[new(default)]
    pub(crate) type_parameters: TypeParameters,
    /// `mode = "..."`, absent when the container's dependencies decide.
    #[new(default)]
    pub(crate) mode: Option<Execution>,
    #[new(default)]
    pub(crate) parent: Option<TypeName>,
    #[new(default)]
    pub(crate) introspection: Introspection,
    /// In the order the fields are declared.
    #[new(default)]
    pub(crate) dependencies: Vec<DependencyDeclaration<F>>,
}
