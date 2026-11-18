//! Adapter: reads one field and its snapfit attributes into a dependency declaration.

use super::darling_models::DependencyAttrMeta;
use super::type_inspector::{
    arc_inner_type, dispatched_trait, parse_dispatch_attribute, type_shape,
};
use crate::adapters::syntax::{BuilderSyntax, FieldDeclaration, FieldSyntax, is_user_attribute};
use crate::domain::{
    AttributeProblem, DependencyDeclaration, DependencyName, DomainError, DomainErrors,
    EnumDispatch, Execution, Lifecycle, TypeName,
};

mod deps;
mod rules;
#[cfg(test)]
mod tests;

use super::build_setting::{WrittenBuild, field_construction};
use deps::{Deps, read_deps};
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
    let build = field_construction(&name, meta.build).map_err(DomainErrors::from);
    let deps = read_deps(&name, meta.deps.0).map_err(DomainErrors::from);
    let (build, deps) = DomainErrors::both(build, deps)?;
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

    let syntax = field_syntax(field, ident, constructor, builder, setters);
    let mut dependency = DependencyDeclaration::new(name, syntax);
    dependency.lifecycle = lifecycle;
    dependency.execution = if meta.is_async {
        Execution::Async
    } else {
        Execution::Sync
    };
    dependency.construction = construction;
    dependency.shape = type_shape(&field.ty);
    dependency.prerequisites = siblings;
    dependency.parent_prerequisites = parents;
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
    builder: Option<BuilderSyntax>,
    setters: Vec<syn::Ident>,
) -> FieldSyntax {
    FieldSyntax {
        ident,
        constructor,
        builder,
        setters,
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
