//! Entity: a container as the user declared it, before snapfit checks or plans it.
//!
//! `C` and `F` are the syntax of the container and of each field, which the
//! domain carries without reading so that the emitter gets them back with
//! each decision.

use derive_new::new;
use strum::EnumIs;

use crate::domain::dispatched_type::{DispatchExtension, DispatchedTypeDeclaration, WrittenType};
use crate::domain::names::{ContainerName, DependencyName, FunctionName, SetterName, TypeName};

mod field_type;
#[cfg(test)]
mod tests;

pub(crate) use field_type::{Implementation, TypeShape};

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
    /// Awaited; for a dependency, its error is returned from the container's `init`.
    Async,
}

/// Whether what builds a value can fail, `fallible = true`.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Fallibility {
    #[default]
    Infallible,
    /// Returns a `Result`, whose error the container's construction returns.
    Fallible,
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
        self.label.clone().unwrap_or_else(|| self.field.as_setter())
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

impl Constructor {
    /// The type's own `new`, whether left out or written `constructor(new)`.
    pub(crate) fn is_new(&self) -> bool {
        match self {
            Self::New => true,
            Self::Named(name) => name.as_str() == "new",
            Self::Default => false,
        }
    }
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

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct DependencyDeclaration<F, T = ()> {
    pub(crate) name: DependencyName,
    pub(crate) syntax: F,
    #[new(default)]
    pub(crate) lifecycle: Lifecycle,
    #[new(default)]
    pub(crate) execution: Execution,
    #[new(default)]
    pub(crate) fallibility: Fallibility,
    /// `build = ...`, absent when the field writes none.
    #[new(default)]
    pub(crate) construction: Option<Construction>,
    #[new(default)]
    pub(crate) shape: TypeShape,
    #[new(default)]
    pub(crate) written_type: WrittenType,
    #[new(default)]
    pub(crate) implementation: Implementation<T>,
    /// The siblings the field is built from: passed to a constructor in order,
    /// by name to a factory, and each to its own setter of a builder.
    #[new(default)]
    pub(crate) prerequisites: Vec<Prerequisite>,
    /// `parent(name)` entries of `deps`, which name a parent container's dependency.
    #[new(default)]
    pub(crate) parent_prerequisites: Vec<DependencyName>,
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContainerDeclaration<C, F, S = (), T = ()> {
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
    pub(crate) dependencies: Vec<DependencyDeclaration<F, T>>,
    /// Each `static_dispatch_for`, in the order written.
    #[new(default)]
    pub(crate) dispatched_types: Vec<DispatchedTypeDeclaration<S>>,
    /// Each `extend_static_dispatch` present in the build, in the order written.
    #[new(default)]
    pub(crate) extensions: Vec<DispatchExtension<T>>,
}
