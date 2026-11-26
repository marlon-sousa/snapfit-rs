//! Adapter: the call that builds a value by a function of its type. Plays the
//! role of its parent.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

use super::{Site, located_at};
use crate::adapters::syntax::FieldSyntax;
use crate::domain::{Constructor, FunctionName};

/// `<T>::new(...)`, `<T as Default>::default()`, or `<T>::connect(...)` with
/// the user's own `connect`, from the field's `build` or the container's
/// `leaf_build`. A call to `new`, or to the function `leaf_build` names, is
/// located at `T`, the field's type or its `implementation`, where a type
/// with no such function, or one that takes another number of arguments,
/// is reported for each field on its own line.
pub(super) fn constructor_call(
    target: &syn::Type,
    constructor: &Constructor,
    field: &FieldSyntax,
    site: Site<'_>,
    args: impl Iterator<Item = TokenStream>,
) -> TokenStream {
    let span = located_at(target.span());
    match constructor {
        Constructor::New => quote_spanned!(span=> <#target>::new(#(#args),*)),
        Constructor::Default => quote!(<#target as ::std::default::Default>::default()),
        Constructor::Named(function) => {
            let function = field.constructor.clone().unwrap_or_else(|| {
                let mut leaf = site
                    .container
                    .leaf_constructor
                    .clone()
                    .unwrap_or_else(|| function_ident(function));
                leaf.set_span(span);
                leaf
            });
            quote_spanned!(span=> <#target>::#function(#(#args),*))
        }
    }
}

pub(super) fn function_ident(function: &FunctionName) -> syn::Ident {
    format_ident!("{}", function.as_str())
}
