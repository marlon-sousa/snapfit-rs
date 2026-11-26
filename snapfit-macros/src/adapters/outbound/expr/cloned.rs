//! Adapter: the clone of a stored singleton, spanned at its field's type.

use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::spanned::Spanned;

use super::located_at;

/// `<Config as Clone>::clone(&self.config)`, so that a type that is not
/// `Clone` is reported at the field that declares it, not at the container.
/// The whole clone is located at the type, so that a dependency of another
/// type than its dependent's function takes is reported there too.
pub(in crate::adapters::outbound) fn cloned(ty: &syn::Type, place: TokenStream) -> TokenStream {
    quote_spanned!(located_at(ty.span())=> <#ty as ::std::clone::Clone>::clone(&#place))
}
