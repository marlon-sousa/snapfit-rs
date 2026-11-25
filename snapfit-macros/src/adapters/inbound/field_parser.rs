//! Adapter: reads one field and its snapfit attributes into a dependency declaration.

use quote::quote;
use syn::ext::IdentExt;

use super::darling_models::DependencyAttrMeta;
use super::type_inspector::{compact_type_tokens, type_shape, written_type};
use crate::adapters::syntax::FieldDeclaration;
use crate::domain::{
    AttributeProblem, DeclarationProblem, DependencyDeclaration, DependencyName, DomainError,
    DomainErrors, Execution, Fallibility, Implementation, Lifecycle, StaticDispatchProblem,
    TypeName, Variant,
};

mod deps;
mod fallible;
mod implementation;
mod rules;
mod settings;
#[cfg(test)]
mod tests;
mod written;

use super::build_setting::WrittenBuild;
use deps::Deps;
use implementation::WrittenImplementation;
use rules::ensure_unconditional;
use settings::{read_settings, unreadable_field_attribute};
use written::{WrittenField, field_syntax};

/// The declaration, or every problem the field's attributes have.
pub(super) fn parse_field(field: &syn::Field) -> Result<FieldDeclaration, DomainErrors> {
    let (name, ident) = field_name(field)?;
    let unconditional = ensure_unconditional(field, &name).map_err(DomainErrors::from);
    let lifecycle = lifecycle_attr(&field.attrs, &name);
    let old_dispatch = refuse_old_dispatch(field, &name).map_err(DomainErrors::from);
    let ((_, lifecycle), _) =
        DomainErrors::both(DomainErrors::both(unconditional, lifecycle), old_dispatch)?;
    let Some((lifecycle, meta)) = lifecycle else {
        return Err(unannotated(&name).into());
    };
    let is_async = meta.is_async;
    let (((build, deps), fallible), implementation) = read_settings(&name, meta)?;
    // The word is kept exactly when the field is fallible: the wrapping is spanned at it.
    let fallibility = if fallible.is_some() {
        Fallibility::Fallible
    } else {
        Fallibility::Infallible
    };
    let WrittenBuild {
        construction,
        constructor,
        builder,
    } = build;
    let Deps {
        siblings,
        parents,
        setters,
    } = deps;

    // The type is kept exactly when one is named: the generated code builds it.
    let (named, implementation) = match implementation {
        Some(WrittenImplementation::One(ty)) => (
            Implementation::Named(implementation_variant(ty.clone())?),
            Some(ty),
        ),
        Some(WrittenImplementation::List(types)) => (implementation_list(types)?, None),
        None => (Implementation::OfField, None),
    };
    let written = WrittenField {
        constructor,
        builder,
        setters,
        fallible,
        implementation,
    };
    let syntax = field_syntax(field, ident, written);
    let mut dependency = DependencyDeclaration::new(name, syntax);
    dependency.lifecycle = lifecycle;
    dependency.execution = if is_async {
        Execution::Async
    } else {
        Execution::Sync
    };
    dependency.fallibility = fallibility;
    dependency.implementation = named;
    dependency.construction = construction;
    dependency.shape = type_shape(&field.ty);
    dependency.written_type = written_type(&field.ty);
    dependency.prerequisites = siblings;
    dependency.parent_prerequisites = parents;
    Ok(dependency)
}

fn field_name(field: &syn::Field) -> Result<(DependencyName, syn::Ident), DomainError> {
    let ident = field.ident.clone().ok_or(DeclarationProblem::TupleStruct)?;
    Ok((DependencyName::new(ident.unraw().to_string())?, ident))
}

fn unannotated(name: &DependencyName) -> DomainError {
    AttributeProblem::Unannotated {
        dependency: name.clone(),
    }
    .into()
}

fn lifecycle_attr(
    attrs: &[syn::Attribute],
    name: &DependencyName,
) -> Result<Option<(Lifecycle, DependencyAttrMeta)>, DomainErrors> {
    let matched: Vec<_> = attrs.iter().filter_map(lifecycle_attribute).collect();
    if matched.len() > 1 {
        let repeated = AttributeProblem::MultipleLifecycles {
            dependency: name.clone(),
        };
        return Err(DomainError::from(repeated).into());
    }
    matched
        .into_iter()
        .next()
        .map(|(lifecycle, attr)| {
            DependencyAttrMeta::from_attribute(attr)
                .map(|meta| (lifecycle, meta))
                .map_err(|unreadable| unreadable_field_attribute(name, unreadable))
        })
        .transpose()
}

/// `#[singleton]` or `#[transient]`, with the lifecycle it declares.
fn lifecycle_attribute(attr: &syn::Attribute) -> Option<(Lifecycle, &syn::Attribute)> {
    if attr.path().is_ident("singleton") {
        Some((Lifecycle::Singleton, attr))
    } else if attr.path().is_ident("transient") {
        Some((Lifecycle::Transient, attr))
    } else {
        None
    }
}

/// `#[dispatch(...)]`, the syntax planned for static dispatch before
/// `static_dispatch_for` replaced it, refused whatever it holds.
fn refuse_old_dispatch(field: &syn::Field, name: &DependencyName) -> Result<(), DomainError> {
    let marked = field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("dispatch"));
    match marked {
        true => Err(StaticDispatchProblem::OldAttribute {
            dependency: name.clone(),
        }
        .into()),
        false => Ok(()),
    }
}

/// A type `implementation` names, by how it is written.
fn implementation_variant(ty: syn::Type) -> Result<Variant<syn::Type>, DomainError> {
    let name = TypeName::new(compact_type_tokens(quote!(#ty)))?;
    Ok(Variant::new(name, ty))
}

fn implementation_list(types: Vec<syn::Type>) -> Result<Implementation<syn::Type>, DomainError> {
    let variants = types
        .into_iter()
        .map(implementation_variant)
        .collect::<Result<_, _>>();
    Ok(Implementation::Listed(variants?))
}
