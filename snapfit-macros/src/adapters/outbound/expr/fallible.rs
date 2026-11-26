//! Adapter: the value of a fallible field, taken out of the `Result` that builds it.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::ext::IdentExt;

use super::Site;
use crate::adapters::outbound::spanned::spanned_runtime;
use crate::adapters::syntax::FieldSyntax;
use crate::domain::Step;

/// The value of the `Result` that `built` returns, or the return of its
/// error, wrapped into `snapfit::BuildError`. Only the wrapping call and
/// the value are spanned at the field's `fallible` word, where a `built`
/// that is no `Result`, or holds a value of the wrong type, is reported.
/// Each local is bound with the span it is used with, so that it resolves
/// when the word and `#[container]` have different hygiene.
/// A `match` stands where a `?` could, so that a value of the wrong type is
/// reported as mismatched types, not against a `?` the user never wrote.
pub(super) fn value(built: TokenStream, step: &Step<FieldSyntax>, site: Site<'_>) -> TokenStream {
    let field = &step.syntax;
    let span = field.fallible.as_ref().unwrap_or(&field.ident).span();
    let container = site.container.ident.unraw().to_string();
    let runtime = spanned_runtime(&site.container.runtime, span);
    let name = field.ident.unraw().to_string();
    let built_local = quote_spanned!(span=> __snapfit_built);
    let value = quote_spanned!(span=> __snapfit_value);
    let wrapped = quote_spanned! {span=>
        #runtime::__private::BuildResult::into_build_result(#built_local, #container, #name)
    };
    quote!(match {
        let #built_local = #built;
        #wrapped
    } {
        ::std::result::Result::Ok(#value) => #value,
        ::std::result::Result::Err(__snapfit_error) => {
            return ::std::result::Result::Err(::std::convert::From::from(__snapfit_error))
        }
    })
}
