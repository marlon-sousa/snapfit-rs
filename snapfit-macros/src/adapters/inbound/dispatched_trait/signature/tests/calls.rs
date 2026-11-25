//! What a method bounds `Self` by, how it is called, and `Self` written through a macro.

use syn::parse_quote;

use super::*;
use crate::domain::SelfBound;

#[test]
fn a_where_clause_bounding_self_only_by_sized_or_a_lifetime_can_be_forwarded() {
    let cases: [(syn::TraitItemFn, SelfBound); 6] = [
        (
            parse_quote!(
                fn f(&self);
            ),
            SelfBound::Forwardable,
        ),
        (
            parse_quote!(
                fn f(self)
                where
                    Self: Sized;
            ),
            SelfBound::Forwardable,
        ),
        (
            parse_quote!(
                fn f(self)
                where
                    Self: ::core::marker::Sized;
            ),
            SelfBound::Forwardable,
        ),
        (
            parse_quote!(
                fn f<'a>(&'a self)
                where
                    Self: 'a + Sized;
            ),
            SelfBound::Forwardable,
        ),
        (
            parse_quote!(
                fn f(&self)
                where
                    Self: Clone;
            ),
            SelfBound::Other,
        ),
        (
            parse_quote!(
                fn f(self)
                where
                    Self: Sized + Clone;
            ),
            SelfBound::Other,
        ),
    ];
    for (method, self_bound) in cases {
        let written = quote::quote!(#method).to_string();
        assert_eq!(shape(method).self_bound, self_bound, "{written}");
    }
}

#[test]
fn an_async_method_is_awaited_and_an_unsafe_one_called_in_an_unsafe_block() {
    let cases: [(syn::TraitItemFn, Execution, Safety); 4] = [
        (
            parse_quote!(
                fn f(&self);
            ),
            Execution::Sync,
            Safety::Safe,
        ),
        (
            parse_quote!(
                async fn f(&self);
            ),
            Execution::Async,
            Safety::Safe,
        ),
        (
            parse_quote!(
                unsafe fn f(&self);
            ),
            Execution::Sync,
            Safety::Unsafe,
        ),
        (
            parse_quote!(
                async unsafe fn f(&self);
            ),
            Execution::Async,
            Safety::Unsafe,
        ),
    ];
    for (method, execution, safety) in cases {
        let written = quote::quote!(#method).to_string();
        let shape = shape(method);
        assert_eq!(
            (shape.execution, shape.safety),
            (execution, safety),
            "{written}"
        );
    }
}

#[test]
fn self_passed_by_a_macro_as_a_type_is_the_whole_return_type() {
    let output = crate::test_support::through_a_macro(quote::quote!(Self));
    let method: syn::TraitItemFn = parse_quote!(fn f(&self) -> #output;);

    assert_eq!(
        shape(method),
        MethodShape::new(
            Receiver::Reference,
            SelfUse::Forwardable,
            SelfBound::Forwardable,
            ReturnShape::ItSelf,
            Execution::Sync,
            Safety::Safe,
        )
    );
}
