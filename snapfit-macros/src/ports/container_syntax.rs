use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::domain::DomainError;

#[cfg(test)]
mod tests;

const SNAPFIT_FIELD_ATTRIBUTES: [&str; 3] = ["singleton", "transient", "dispatch"];

/// The user's own syntax for one container field, kept beside the pure domain
/// IR so the emitter can reproduce it exactly (types, visibility, attributes)
/// and diagnostics can point at it.
#[derive(Debug, Clone)]
pub struct FieldSyntax {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub vis: syn::Visibility,
    pub attrs: Vec<syn::Attribute>,
    /// The tokens a diagnostic about this field underlines: its snapfit
    /// attribute, or its name when it has none.
    pub origin: TokenStream,
}

/// The user's own syntax for the container struct.
#[derive(Debug, Clone)]
pub struct ContainerSyntax {
    pub ident: syn::Ident,
    pub vis: syn::Visibility,
    pub attrs: Vec<syn::Attribute>,
    pub generics: syn::Generics,
    pub fields: Vec<FieldSyntax>,
}

impl ContainerSyntax {
    pub fn new(
        ident: &syn::Ident,
        vis: &syn::Visibility,
        attrs: &[syn::Attribute],
        generics: &syn::Generics,
        fields: &syn::Fields,
    ) -> Self {
        Self {
            ident: ident.clone(),
            vis: vis.clone(),
            attrs: attrs.to_vec(),
            generics: generics.clone(),
            fields: fields.iter().filter_map(FieldSyntax::from_field).collect(),
        }
    }

    pub fn field(&self, name: &str) -> Option<&FieldSyntax> {
        self.fields.iter().find(|field| field.ident == name)
    }

    /// A compiler diagnostic for `error`, underlining the offending field's
    /// snapfit attribute, or the container's name when no field is at fault.
    pub fn error_for(&self, error: DomainError) -> syn::Error {
        match error.subject().and_then(|name| self.field(name.as_str())) {
            Some(field) => syn::Error::new_spanned(&field.origin, error),
            None => syn::Error::new(self.ident.span(), error),
        }
    }
}

impl FieldSyntax {
    fn from_field(field: &syn::Field) -> Option<Self> {
        let ident = field.ident.clone()?;
        let origin = field
            .attrs
            .iter()
            .find(|attr| is_snapfit_attribute(attr))
            .map_or_else(|| ident.to_token_stream(), ToTokens::to_token_stream);
        let attrs = field
            .attrs
            .iter()
            .filter(|attr| !is_snapfit_attribute(attr))
            .cloned()
            .collect();
        Some(Self {
            ident,
            ty: field.ty.clone(),
            vis: field.vis.clone(),
            attrs,
            origin,
        })
    }
}

fn is_snapfit_attribute(attr: &syn::Attribute) -> bool {
    SNAPFIT_FIELD_ATTRIBUTES
        .iter()
        .any(|name| attr.path().is_ident(name))
}
