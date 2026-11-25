//! Adapter: the clone of a stored singleton, spanned at its field's type.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// `<Config as Clone>::clone(&self.config)`, so that a type that is not
/// `Clone` is reported at the field that declares it, not at the container.
pub(in crate::adapters::outbound) fn cloned(ty: &syn::Type, place: TokenStream) -> TokenStream {
    let clone = quote_spanned!(ty.span()=> <#ty as ::std::clone::Clone>::clone);
    quote!(#clone(&#place))
}
