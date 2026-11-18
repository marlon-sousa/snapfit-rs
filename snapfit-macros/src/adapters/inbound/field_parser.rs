//! Adapter: reads one field and its snapfit attributes into a dependency declaration.

use super::darling_models::{DependencyAttrMeta, PrerequisiteMeta};
use super::type_inspector::{
    arc_inner_type, dispatched_trait, parse_dispatch_attribute, type_shape,
};
use crate::adapters::syntax::{FieldDeclaration, FieldSyntax, is_user_attribute};
use crate::domain::{
    AttributeProblem, DependencyDeclaration, DependencyName, DomainError, DomainErrors,
    EnumDispatch, Execution, Lifecycle, TypeName,
};

mod rules;
#[cfg(test)]
mod tests;

use super::build_setting::field_construction;
use rules::{attribute_error, ensure_unconditional};

/// The declaration, or every problem the field's attributes have.
pub(super) fn parse_field(field: &syn::Field) -> Result<FieldDeclaration, DomainErrors> {
    let (name, ident) = field_name(field)?;
    let unconditional = ensure_unconditional(field, &name).map_err(DomainErrors::from);
    let lifecycle = lifecycle_attr(&field.attrs, &name).map_err(DomainErrors::from);
    let dispatch = dispatch_attr(field, &name).map_err(DomainErrors::from);
    let ((_, lifecycle), dispatch) =
        DomainErrors::both(DomainErrors::both(unconditional, lifecycle), dispatch)?;
    let (lifecycle, meta) = match (lifecycle, &dispatch) {
        (Some(found), _) => found,
        (None, Some(_)) => (Lifecycle::Singleton, DependencyAttrMeta::default()),
        (None, None) => return Err(unannotated(&name).into()),
    };
    let construction = field_construction(&name, meta.build).map_err(DomainErrors::from);
    let prerequisites = prerequisites_of(&name, meta.deps.0).map_err(DomainErrors::from);
    let ((construction, constructor), (prerequisites, parent_prerequisites)) =
        DomainErrors::both(construction, prerequisites)?;

    let mut dependency = DependencyDeclaration::new(name, field_syntax(field, ident, constructor));
    dependency.lifecycle = lifecycle;
    dependency.execution = if meta.is_async {
        Execution::Async
    } else {
        Execution::Sync
    };
    dependency.construction = construction;
    dependency.shape = type_shape(&field.ty);
    dependency.prerequisites = prerequisites;
    dependency.parent_prerequisites = parent_prerequisites;
    dependency.dispatch = dispatch;
    Ok(dependency)
}

fn field_name(field: &syn::Field) -> Result<(DependencyName, syn::Ident), DomainError> {
    let ident = field.ident.clone().ok_or(DomainError::TupleStruct)?;
    Ok((DependencyName::new(ident.to_string())?, ident))
}

fn field_syntax(
    field: &syn::Field,
    ident: syn::Ident,
    constructor: Option<syn::Ident>,
) -> FieldSyntax {
    FieldSyntax {
        ident,
        constructor,
        ty: field.ty.clone(),
        arc_inner: arc_inner_type(&field.ty).unwrap_or(&field.ty).clone(),
        vis: field.vis.clone(),
        attrs: field
            .attrs
            .iter()
            .filter(is_user_attribute)
            .cloned()
            .collect(),
    }
}

fn unannotated(name: &DependencyName) -> DomainError {
    DomainError::Unannotated {
        dependency: name.clone(),
    }
}

fn lifecycle_attr(
    attrs: &[syn::Attribute],
    name: &DependencyName,
) -> Result<Option<(Lifecycle, DependencyAttrMeta)>, DomainError> {
    let matched: Vec<_> = attrs.iter().filter_map(lifecycle_attribute).collect();
    if matched.len() > 1 {
        return Err(DomainError::MultipleLifecycles {
            dependency: name.clone(),
        });
    }
    matched
        .into_iter()
        .next()
        .map(|(lifecycle, attr)| {
            DependencyAttrMeta::from_attribute(attr)
                .map(|meta| (lifecycle, meta))
                .map_err(|problem| attribute_error(name, problem))
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

fn dispatch_attr(
    field: &syn::Field,
    name: &DependencyName,
) -> Result<Option<EnumDispatch>, DomainError> {
    let matched: Vec<_> = field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("dispatch"))
        .collect();
    if matched.len() > 1 {
        return Err(DomainError::MultipleDispatches {
            dependency: name.clone(),
        });
    }
    matched
        .first()
        .map(|attr| {
            enum_dispatch(attr, &field.ty).map_err(|problem| attribute_error(name, problem))
        })
        .transpose()
}

fn enum_dispatch(attr: &syn::Attribute, ty: &syn::Type) -> Result<EnumDispatch, AttributeProblem> {
    let implementations = parse_dispatch_attribute(attr)?
        .into_iter()
        .map(type_name)
        .collect::<Result<_, _>>()?;
    let trait_name = type_name(dispatched_trait(ty)?)?;
    Ok(EnumDispatch::new(trait_name, implementations))
}

fn type_name(text: String) -> Result<TypeName, AttributeProblem> {
    TypeName::new(text).map_err(|_| AttributeProblem::BlankName)
}

/// Siblings and `parent(...)` entries of `deps`, each in the order written.
fn prerequisites_of(
    name: &DependencyName,
    entries: Vec<PrerequisiteMeta>,
) -> Result<(Vec<DependencyName>, Vec<DependencyName>), DomainError> {
    let (siblings, parents): (Vec<_>, Vec<_>) =
        entries.into_iter().partition(PrerequisiteMeta::is_sibling);
    let siblings = dependency_names(siblings).map_err(|problem| attribute_error(name, problem))?;
    let parents = dependency_names(parents).map_err(|problem| attribute_error(name, problem))?;
    Ok((siblings, parents))
}

fn dependency_names(
    entries: Vec<PrerequisiteMeta>,
) -> Result<Vec<DependencyName>, AttributeProblem> {
    entries
        .into_iter()
        .map(PrerequisiteMeta::into_name)
        .map(DependencyName::new)
        .collect::<Result<_, _>>()
        .map_err(|_| AttributeProblem::BlankName)
}
