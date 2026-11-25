//! Adapter: reads the user's struct, through syn and darling, into a container
//! declaration, and the item `#[snapfit::static_dispatch]` is on into a
//! dispatch attribute.

mod build_setting;
mod darling_models;
mod dispatch_declarations;
mod dispatched_trait;
mod field_parser;
mod runtime_path;
mod type_inspector;

#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;

pub(crate) use dispatched_trait::read_dispatch_attribute;
pub(crate) use runtime_path::readable_runtime_path;

use crate::adapters::syntax::{
    ContainerSyntax, Declaration, FieldDeclaration, ReadableParts, is_container_attribute,
    is_user_attribute,
};
use crate::domain::{
    ArgumentProblem, AttributeProblem, Constructor, ContainerDeclaration, ContainerName,
    DeclarationProblem, DomainError, DomainErrors, Execution, Introspection, TypeName,
    TypeParameters,
};
use darling_models::{ContainerAttrMeta, UnreadableAttribute};
use dispatch_declarations::{read_dispatch_declarations, read_readable_dispatch_declarations};

/// `#[container(args)]` on `item`.
pub(crate) fn parse_attribute(
    args: TokenStream,
    item: &syn::ItemStruct,
) -> Result<Declaration, DomainErrors> {
    let header = container_meta(args, &item.attrs).and_then(|meta| container_header(item, meta));
    let dispatch = read_dispatch_declarations(&item.attrs);
    let fields = parse_fields(&item.fields);
    let ((mut container, (dispatched_types, extensions)), dependencies) =
        DomainErrors::both(DomainErrors::both(header, dispatch), fields)?;
    container.dependencies = dependencies;
    container.dispatched_types = dispatched_types;
    container.extensions = extensions;
    Ok(container)
}

/// The [`ReadableParts`] of `item`.
pub(crate) fn read_readable_parts(item: &syn::ItemStruct) -> ReadableParts {
    let (declarations, extensions) = read_readable_dispatch_declarations(&item.attrs);
    let fields = item
        .fields
        .iter()
        .map(field_parser::parse_field)
        .filter_map(Result::ok)
        .collect();
    ReadableParts::new(declarations, extensions, fields)
}

/// The container's name and settings, or every setting that cannot be read.
fn container_header(
    item: &syn::ItemStruct,
    meta: ContainerAttrMeta,
) -> Result<Declaration, DomainErrors> {
    let name = ContainerName::new(item.ident.to_string()).map_err(DomainErrors::from);
    let introspection = meta.introspection;
    let (name, settings) = DomainErrors::both(name, container_settings(meta))?;
    let mut container = ContainerDeclaration::new(
        name,
        container_syntax(item, settings.leaf_ident, settings.runtime),
    );
    container.type_parameters = type_parameters(&item.generics);
    container.mode = settings.mode;
    container.parent = settings.parent;
    container.introspection = if introspection {
        Introspection::On
    } else {
        Introspection::Off
    };
    container.leaf_constructor = settings.leaf_constructor;
    Ok(container)
}

/// The settings of `#[container(...)]` that are checked when they are read.
struct ContainerSettings {
    mode: Option<Execution>,
    parent: Option<TypeName>,
    leaf_constructor: Constructor,
    /// The function of `leaf_build = constructor(...)` as the user wrote it.
    leaf_ident: Option<syn::Ident>,
    runtime: syn::Path,
}

/// The container's `mode`, `parent` and `leaf_build`, or every problem they have.
/// Each setting must be checked apart from the others: one rescued from an
/// attribute that does not read is checked alone.
fn container_settings(meta: ContainerAttrMeta) -> Result<ContainerSettings, DomainErrors> {
    let mode = meta
        .mode
        .as_deref()
        .map(execution_of)
        .transpose()
        .map_err(DomainErrors::from);
    let parent = meta
        .parent
        .map(|parent| TypeName::new(parent.0))
        .transpose()
        .map_err(DomainErrors::from);
    let leaf = build_setting::leaf_constructor(meta.leaf_build).map_err(DomainErrors::from);
    let runtime = runtime_path::runtime_path(meta.runtime).map_err(|problem| {
        DomainErrors::from(DomainError::from(AttributeProblem::Container { problem }))
    });
    let (mode, (parent, ((leaf_constructor, leaf_ident), runtime))) = DomainErrors::both(
        mode,
        DomainErrors::both(parent, DomainErrors::both(leaf, runtime)),
    )?;
    Ok(ContainerSettings {
        mode,
        parent,
        leaf_constructor,
        leaf_ident,
        runtime,
    })
}

fn container_syntax(
    item: &syn::ItemStruct,
    leaf_constructor: Option<syn::Ident>,
    runtime: syn::Path,
) -> ContainerSyntax {
    ContainerSyntax {
        ident: item.ident.clone(),
        leaf_constructor,
        runtime,
        vis: item.vis.clone(),
        attrs: item
            .attrs
            .iter()
            .filter(is_user_attribute)
            .cloned()
            .collect(),
    }
}

fn type_parameters(generics: &syn::Generics) -> TypeParameters {
    if generics.params.is_empty() && generics.where_clause.is_none() {
        TypeParameters::None
    } else {
        TypeParameters::Declared
    }
}

/// `mode = "..."`, read by the adapter so that its error names the mode
/// without darling's location.
fn execution_of(mode: &str) -> Result<Execution, DomainError> {
    match mode {
        "sync" => Ok(Execution::Sync),
        "async" => Ok(Execution::Async),
        other => Err(AttributeProblem::Container {
            problem: ArgumentProblem::InvalidMode(other.to_string()),
        }
        .into()),
    }
}

/// The settings are the macro's own arguments; another `#[container]` left
/// on the struct is a second declaration.
fn container_meta(
    args: TokenStream,
    attrs: &[syn::Attribute],
) -> Result<ContainerAttrMeta, DomainErrors> {
    if attrs.iter().any(is_container_attribute) {
        Err(DeclarationProblem::MultipleContainerAttributes)?;
    }
    ContainerAttrMeta::from_tokens(args).map_err(unreadable_container_attribute)
}

/// The attribute's problem, then the problems of each setting that reads on its own.
fn unreadable_container_attribute(
    unreadable: UnreadableAttribute<ContainerAttrMeta>,
) -> DomainErrors {
    let UnreadableAttribute { problem, settings } = unreadable;
    let found = settings.into_iter().map(container_settings);
    let rest = found.filter_map(Result::err).flatten();
    DomainErrors::new(
        AttributeProblem::Container { problem }.into(),
        rest.collect(),
    )
}

fn parse_fields(fields: &syn::Fields) -> Result<Vec<FieldDeclaration>, DomainErrors> {
    match fields {
        syn::Fields::Named(named) => {
            DomainErrors::collect(named.named.iter().map(field_parser::parse_field))
        }
        syn::Fields::Unit => Ok(Vec::new()),
        syn::Fields::Unnamed(_) => Err(DeclarationProblem::TupleStruct)?,
    }
}
