//! Adapter: the compiler errors for every problem found, each underlining
//! the field it is about, and the struct kept as written when expansion fails.

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

use crate::adapters::syntax::{is_snapfit_attribute, is_user_attribute};
use crate::domain::{DomainError, DomainErrors};

#[cfg(test)]
mod tests;

/// The tokens a diagnostic underlines: a field's snapfit attribute, or its
/// name when it has none, and the container's name for the rest.
#[derive(Debug, Clone)]
pub(crate) struct Origins {
    container: Span,
    fields: Vec<FieldOrigin>,
}

#[derive(Debug, Clone)]
struct FieldOrigin {
    name: String,
    tokens: TokenStream,
}

impl Origins {
    pub(crate) fn new(ident: &syn::Ident, fields: &syn::Fields) -> Self {
        Self {
            container: ident.span(),
            fields: fields.iter().filter_map(FieldOrigin::of).collect(),
        }
    }

    /// One `compile_error!` per problem, in the order they were found.
    pub(crate) fn compile_errors(&self, errors: &DomainErrors) -> TokenStream {
        errors
            .iter()
            .map(|error| self.error_for(error).to_compile_error())
            .collect()
    }

    fn error_for(&self, error: &DomainError) -> syn::Error {
        let field = error
            .subject()
            .and_then(|name| self.fields.iter().find(|field| field.name == name.as_str()));
        match field {
            Some(field) => syn::Error::new_spanned(&field.tokens, error),
            None => syn::Error::new(self.container, error),
        }
    }
}

impl FieldOrigin {
    fn of(field: &syn::Field) -> Option<Self> {
        let ident = field.ident.as_ref()?;
        let tokens = field
            .attrs
            .iter()
            .find(|attr| is_snapfit_attribute(attr))
            .map_or_else(|| ident.to_token_stream(), ToTokens::to_token_stream);
        Some(Self {
            name: ident.to_string(),
            tokens,
        })
    }
}

/// The struct without snapfit's attributes, so that a failed expansion shows
/// the real errors instead of follow-on "cannot find type" errors.
pub(crate) fn fallback_struct(item: &syn::ItemStruct) -> TokenStream {
    let mut item = item.clone();
    for field in &mut item.fields {
        field.attrs.retain(|attr| is_user_attribute(&attr));
    }
    item.to_token_stream()
}
