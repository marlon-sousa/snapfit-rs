//! Adapter: reads the user's struct, through syn and darling, into a container declaration.

mod darling_models;
mod field_parser;
mod type_inspector;

#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;

use crate::adapters::syntax::{
    ContainerSyntax, Declaration, FieldDeclaration, is_container_attribute, is_user_attribute,
};
use crate::domain::{
    AttributeProblem, ContainerDeclaration, ContainerName, DomainError, DomainErrors, Execution,
    Introspection, TypeName, TypeParameters,
};
use darling_models::ContainerAttrMeta;

/// `#[container(args)]` on `item`.
pub(crate) fn parse_attribute(
    args: TokenStream,
    item: &syn::ItemStruct,
) -> Result<Declaration, DomainErrors> {
    let header = container_meta(args, &item.attrs)
        .and_then(|meta| container_header(item, meta))
        .map_err(DomainErrors::from);
    let (mut container, dependencies) = DomainErrors::both(header, parse_fields(&item.fields))?;
    container.dependencies = dependencies;
    Ok(container)
}

fn container_header(
    item: &syn::ItemStruct,
    meta: ContainerAttrMeta,
) -> Result<Declaration, DomainError> {
    let name = ContainerName::new(item.ident.to_string())?;
    let mut container = ContainerDeclaration::new(name, container_syntax(item));
    container.type_parameters = type_parameters(&item.generics);
    container.mode = meta.mode.as_deref().map(execution_of).transpose()?;
    container.parent = meta
        .parent
        .map(|parent| TypeName::new(parent.0))
        .transpose()?;
    container.introspection = if meta.introspection {
        Introspection::On
    } else {
        Introspection::Off
    };
    Ok(container)
}

fn container_syntax(item: &syn::ItemStruct) -> ContainerSyntax {
    ContainerSyntax {
        ident: item.ident.clone(),
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
        other => Err(DomainError::ContainerAttribute {
            problem: AttributeProblem::InvalidMode(other.to_string()),
        }),
    }
}

/// The settings are the macro's own arguments; another `#[container]` left
/// on the struct is a second declaration.
fn container_meta(
    args: TokenStream,
    attrs: &[syn::Attribute],
) -> Result<ContainerAttrMeta, DomainError> {
    if attrs.iter().any(is_container_attribute) {
        return Err(DomainError::MultipleContainerAttributes);
    }
    ContainerAttrMeta::from_tokens(args).map_err(container_attribute)
}

fn container_attribute(problem: AttributeProblem) -> DomainError {
    DomainError::ContainerAttribute { problem }
}

fn parse_fields(fields: &syn::Fields) -> Result<Vec<FieldDeclaration>, DomainErrors> {
    match fields {
        syn::Fields::Named(named) => {
            DomainErrors::collect(named.named.iter().map(field_parser::parse_field))
        }
        syn::Fields::Unit => Ok(Vec::new()),
        syn::Fields::Unnamed(_) => Err(DomainError::TupleStruct.into()),
    }
}
