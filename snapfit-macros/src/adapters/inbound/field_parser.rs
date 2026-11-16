use quote::quote;

use super::darling_models::DependencyAttrMeta;
use super::type_inspector::{
    compact_type_tokens, parse_dispatch_attribute, resolve_trait_dispatch,
};
use crate::domain::{
    DependencyIR, DependencyName, DomainError, Lifecycle, TraitDispatch, TypeName,
};

#[cfg(test)]
mod tests;

pub(super) fn parse_field(field: &syn::Field) -> Result<DependencyIR, DomainError> {
    let name = extract_field_name(field)?;
    let ty = &field.ty;
    let type_string = TypeName::new(compact_type_tokens(quote!(#ty)))?;
    let lifecycle_entry = extract_lifecycle_attr(&field.attrs, &name)?;
    let dispatch_impls = extract_dispatch_attr(&field.attrs, &name)?;

    if lifecycle_entry.is_none() && dispatch_impls.is_none() {
        return Err(DomainError::InvalidConfiguration(format!(
            "field '{name}' must be annotated with #[singleton], #[transient], or #[dispatch]"
        )));
    }

    let (lifecycle, meta) =
        lifecycle_entry.unwrap_or((Lifecycle::Singleton, DependencyAttrMeta::default()));
    let trait_dispatch = resolve_trait_dispatch(&field.ty, dispatch_impls)?;
    build_dependency_ir(name, type_string, lifecycle, meta, trait_dispatch)
}

fn extract_field_name(field: &syn::Field) -> Result<DependencyName, DomainError> {
    let ident = field.ident.as_ref().ok_or_else(|| {
        DomainError::InvalidConfiguration("container fields must be named".to_string())
    })?;
    DependencyName::new(ident.to_string())
}

fn extract_lifecycle_attr(
    attrs: &[syn::Attribute],
    field_name: &DependencyName,
) -> Result<Option<(Lifecycle, DependencyAttrMeta)>, DomainError> {
    let matched: Vec<_> = attrs
        .iter()
        .filter_map(|attr| match_lifecycle(attr).map(|lc| (lc, attr)))
        .collect();

    if matched.len() > 1 {
        return Err(DomainError::InvalidConfiguration(format!(
            "field '{field_name}' cannot declare multiple lifecycle attributes"
        )));
    }

    matched
        .into_iter()
        .next()
        .map(|(lifecycle, attr)| Ok((lifecycle, DependencyAttrMeta::from_attribute(attr)?)))
        .transpose()
}

fn match_lifecycle(attr: &syn::Attribute) -> Option<Lifecycle> {
    if attr.path().is_ident("singleton") {
        Some(Lifecycle::Singleton)
    } else if attr.path().is_ident("transient") {
        Some(Lifecycle::Transient)
    } else {
        None
    }
}

fn extract_dispatch_attr(
    attrs: &[syn::Attribute],
    field_name: &DependencyName,
) -> Result<Option<Vec<String>>, DomainError> {
    let matched: Vec<_> = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("dispatch"))
        .collect();

    if matched.len() > 1 {
        return Err(DomainError::InvalidConfiguration(format!(
            "field '{field_name}' cannot declare multiple #[dispatch] attributes"
        )));
    }

    matched
        .into_iter()
        .next()
        .map(parse_dispatch_attribute)
        .transpose()
}

fn build_dependency_ir(
    name: DependencyName,
    type_string: TypeName,
    lifecycle: Lifecycle,
    meta: DependencyAttrMeta,
    trait_dispatch: TraitDispatch,
) -> Result<DependencyIR, DomainError> {
    if lifecycle.is_transient() && meta.is_async {
        return Err(DomainError::InvalidConfiguration(format!(
            "transient dependency '{name}' cannot be async because getters are synchronous"
        )));
    }
    if meta.constructor.is_some() && meta.builder.is_some() {
        return Err(DomainError::InvalidConfiguration(format!(
            "dependency '{name}' cannot specify both 'constructor' and 'builder'"
        )));
    }

    let constructor_name = validate_optional_ident(meta.constructor, "constructor")?
        .unwrap_or_else(|| "new".to_string());
    let custom_builder = validate_optional_ident(meta.builder, "builder")?;
    let prerequisites = meta
        .deps
        .0
        .into_iter()
        .map(DependencyName::new)
        .collect::<Result<Vec<_>, _>>()?;

    let mut dep = DependencyIR::new(name, type_string);
    dep.prerequisites = prerequisites;
    dep.lifecycle = lifecycle;
    dep.is_async = meta.is_async;
    dep.constructor_name = constructor_name;
    dep.custom_builder = custom_builder;
    dep.trait_dispatch = trait_dispatch;
    Ok(dep)
}

fn validate_optional_ident(
    value: Option<String>,
    label: &str,
) -> Result<Option<String>, DomainError> {
    value
        .map(|s| {
            (!s.trim().is_empty()).then_some(s).ok_or_else(|| {
                DomainError::InvalidConfiguration(format!("{label} name cannot be empty or blank"))
            })
        })
        .transpose()
}
