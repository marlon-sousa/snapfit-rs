//! Test support: the dispatched types, extensions and fields that the tests
//! of the static dispatch policy build, whose syntax is the text they are
//! written as.

use crate::domain::{
    AttributePosition, ContainerDeclaration, ContainerName, DependencyDeclaration, DependencyName,
    DispatchExtension, DispatchedTypeDeclaration, Implementation, TypeName, Variant, WrittenType,
};

pub(crate) type Container = ContainerDeclaration<(), (), &'static str, &'static str>;

pub(crate) fn type_name(text: &str) -> TypeName {
    TypeName::new(text).unwrap()
}

pub(crate) fn variant(text: &'static str) -> Variant<&'static str> {
    Variant::new(type_name(text), text)
}

pub(crate) fn declared(
    name: &'static str,
    position: usize,
) -> DispatchedTypeDeclaration<&'static str> {
    DispatchedTypeDeclaration::new(type_name(name), AttributePosition(position), name)
}

pub(crate) fn extended(
    name: &str,
    position: usize,
    variants: &[&'static str],
) -> DispatchExtension<&'static str> {
    DispatchExtension::new(
        type_name(name),
        AttributePosition(position),
        variants.iter().copied().map(variant).collect(),
    )
}

/// A field named `field` whose value is written `written`, naming `implementation` if any.
pub(crate) fn dispatched_field(
    field: &str,
    written: WrittenType,
    implementation: Option<&'static str>,
) -> DependencyDeclaration<(), &'static str> {
    let mut dependency = DependencyDeclaration::new(DependencyName::new(field).unwrap(), ());
    dependency.written_type = written;
    dependency.implementation = implementation.map_or(Implementation::OfField, |text| {
        Implementation::Named(variant(text))
    });
    dependency
}

/// A field named `field` whose value is written `written`, listing `implementations`.
pub(crate) fn listing_field(
    field: &str,
    written: WrittenType,
    implementations: &[&'static str],
) -> DependencyDeclaration<(), &'static str> {
    let mut dependency = dispatched_field(field, written, None);
    dependency.implementation =
        Implementation::Listed(implementations.iter().copied().map(variant).collect());
    dependency
}

pub(crate) fn dispatching_container(
    dependencies: Vec<DependencyDeclaration<(), &'static str>>,
    dispatched_types: Vec<DispatchedTypeDeclaration<&'static str>>,
    extensions: Vec<DispatchExtension<&'static str>>,
) -> Container {
    let mut container = ContainerDeclaration::new(ContainerName::new("App").unwrap(), ());
    container.dependencies = dependencies;
    container.dispatched_types = dispatched_types;
    container.extensions = extensions;
    container
}

pub(crate) fn written_name(text: &str) -> WrittenType {
    WrittenType::Name(type_name(text))
}
