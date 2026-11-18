//! Entity: a container as the user declared it, before snapfit checks or plans it.
//!
//! `C` and `F` are the syntax of the container and of each field, which the
//! domain carries without reading so that the emitter gets them back with
//! each decision.

use derive_new::new;
use strum::EnumIs;

use crate::domain::names::{ContainerName, DependencyName, FunctionName, SetterName, TypeName};

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

#[derive(EnumIs, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Construction {
    /// An associated function of the dependency's type.
    Constructor(Constructor),
    /// The `build_<field>` method the user writes in the container's factories trait.
    Factory,
    /// The type's builder, with one setter per prerequisite.
    Builder(BuilderMethods),
}

/// The functions a builder chain starts and finishes with.
#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuilderMethods {
    /// The type's associated function that returns its builder.
    pub(crate) start: FunctionName,
    /// The builder's method that returns the value.
    pub(crate) finish: FunctionName,
}

/// One sibling entry of `deps = [...]`.
#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Prerequisite {
    /// The container's field the value comes from.
    pub(crate) field: DependencyName,
    /// `disk` in `disk = folder_files`: the builder setter that takes it, when
    /// it is not the field's own name.
    #[new(default)]
    pub(crate) label: Option<SetterName>,
}

impl Prerequisite {
    /// The builder setter that takes this value: its label, or the field's name.
    pub(crate) fn setter(&self) -> SetterName {
        self.label.clone().unwrap_or_else(|| self.field.as_kind())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Constructor {
    #[default]
    New,
    /// `Default::default()`, which takes no prerequisites.
    Default,
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
    /// `build = ...`, absent when the field writes none.
    #[new(default)]
    pub(crate) construction: Option<Construction>,
    #[new(default)]
    pub(crate) shape: TypeShape,
    /// The siblings the field is built from: passed to a constructor in order,
    /// by name to a factory, and each to its own setter of a builder.
    #[new(default)]
    pub(crate) prerequisites: Vec<Prerequisite>,
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
    pub(crate) type_parameters: TypeParameters,
    /// `mode = "..."`, absent when the container's dependencies decide.
    #[new(default)]
    pub(crate) mode: Option<Execution>,
    #[new(default)]
    pub(crate) parent: Option<TypeName>,
    #[new(default)]
    pub(crate) introspection: Introspection,
    /// `leaf_build = ...`: how a field with no `deps` and no `build` of its own is built.
    #[new(default)]
    pub(crate) leaf_constructor: Constructor,
    /// In the order the fields are declared.
    #[new(default)]
    pub(crate) dependencies: Vec<DependencyDeclaration<F>>,
}
