//! Adapter: reads the user's struct, through syn and darling, into a container declaration.

mod darling_models;
mod field_parser;
mod type_inspector;

#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;

use crate::adapters::syntax::{ContainerSyntax, Declaration, FieldDeclaration, is_user_attribute};
use crate::domain::{
    AttributeProblem, ContainerDeclaration, ContainerName, DomainError, DomainErrors, Execution,
    Introspection, MacroForm, TypeName, TypeParameters,
};
use darling_models::ContainerAttrMeta;

/// `#[container(args)]` on `item`.
pub(crate) fn parse_attribute(
    args: TokenStream,
    item: &syn::ItemStruct,
) -> Result<Declaration, DomainErrors> {
    let meta = container_meta(args, &item.attrs);
    let parts = StructParts {
        ident: &item.ident,
        vis: &item.vis,
        attrs: &item.attrs,
        generics: &item.generics,
    };
    parse(MacroForm::Attribute, parts, meta, &item.fields)
}

/// `#[derive(Container)]` on `input`.
pub(crate) fn parse_derive(input: &syn::DeriveInput) -> Result<Declaration, DomainErrors> {
    let syn::Data::Struct(data) = &input.data else {
        return Err(DomainError::NotAStruct.into());
    };
    let meta = container_meta(TokenStream::new(), &input.attrs);
    let parts = StructParts {
        ident: &input.ident,
        vis: &input.vis,
        attrs: &input.attrs,
        generics: &input.generics,
    };
    parse(MacroForm::Derive, parts, meta, &data.fields)
}

/// What the attribute and the derive both give, apart from the fields.
struct StructParts<'a> {
    ident: &'a syn::Ident,
    vis: &'a syn::Visibility,
    attrs: &'a [syn::Attribute],
    generics: &'a syn::Generics,
}

fn parse(
    form: MacroForm,
    parts: StructParts<'_>,
    meta: Result<ContainerAttrMeta, DomainError>,
    fields: &syn::Fields,
) -> Result<Declaration, DomainErrors> {
    let header = meta
        .and_then(|meta| container_header(&parts, meta))
        .map_err(DomainErrors::from);
    let (mut container, dependencies) = DomainErrors::both(header, parse_fields(fields))?;
    container.form = form;
    container.dependencies = dependencies;
    Ok(container)
}

fn container_header(
    parts: &StructParts<'_>,
    meta: ContainerAttrMeta,
) -> Result<Declaration, DomainError> {
    let name = ContainerName::new(parts.ident.to_string())?;
    let mut container = ContainerDeclaration::new(name, container_syntax(parts));
    container.type_parameters = type_parameters(parts.generics);
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

fn container_syntax(parts: &StructParts<'_>) -> ContainerSyntax {
    ContainerSyntax {
        ident: parts.ident.clone(),
        vis: parts.vis.clone(),
        attrs: parts
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

fn container_meta(
    args: TokenStream,
    attrs: &[syn::Attribute],
) -> Result<ContainerAttrMeta, DomainError> {
    let container_attrs: Vec<_> = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("container"))
        .collect();
    match (args.is_empty(), container_attrs.as_slice()) {
        (false, []) => ContainerAttrMeta::from_tokens(args).map_err(container_attribute),
        (true, []) => Ok(ContainerAttrMeta::default()),
        (true, [attr]) => ContainerAttrMeta::from_attribute(attr).map_err(container_attribute),
        _ => Err(DomainError::MultipleContainerAttributes),
    }
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
