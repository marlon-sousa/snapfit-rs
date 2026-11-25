use quote::quote;

use super::*;
use crate::test_support::compact;

/// The messages of the `compile_error!` calls in `tokens`, in order.
fn compile_errors(tokens: &TokenStream) -> Vec<String> {
    let file: syn::File = syn::parse2(tokens.clone()).unwrap();
    file.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Macro(item) => Some(
                syn::parse2::<syn::LitStr>(item.mac.tokens.clone())
                    .unwrap()
                    .value(),
            ),
            _ => None,
        })
        .collect()
}

#[test]
fn the_trait_is_kept_as_written_and_followed_by_its_two_impls() {
    let code = compact(static_dispatch_attribute(
        quote!(),
        quote!(
            pub trait Database {
                fn find(&self, id: u32) -> Option<Order>;
            }
        ),
    ));

    assert!(code.starts_with(&compact(quote!(
        pub trait Database {
            fn find(&self, id: u32) -> Option<Order>;
        }
    ))));
    assert!(code.contains("for::snapfit::__private::Either<__SnapfitFirst,__SnapfitRest>"));
    assert!(code.contains("for::snapfit::__private::Only<__SnapfitLast>"));
}

#[test]
fn a_refused_trait_is_kept_as_written_beside_every_error_and_gets_no_impl() {
    let tokens = static_dispatch_attribute(
        quote!(remote = other::Database),
        quote!(
            pub trait Database {
                const LIMIT: usize;
                fn create() -> Self;
            }
        ),
    );

    assert_eq!(
        compile_errors(&tokens),
        [
            "#[snapfit::static_dispatch(remote = ...)] is planned for a future version of snapfit and is not implemented yet",
            "associated const 'LIMIT' in trait 'Database' could have a different value in each implementation, so static dispatch cannot forward it; make it a method instead, such as fn limit(&self) -> usize",
            "method 'create' in trait 'Database' has no receiver, so a dispatched value has no implementation to call it on; take self, &self or &mut self, or move the function out of the trait",
        ]
    );
    let code = compact(&tokens);
    assert!(code.contains(&compact(quote!(pub trait Database))));
    assert!(!code.contains("__private::Either"));
}

#[test]
fn the_attribute_on_a_struct_keeps_the_struct_beside_its_error() {
    let tokens = static_dispatch_attribute(
        quote!(),
        quote!(
            pub struct Database;
        ),
    );

    assert_eq!(
        compile_errors(&tokens),
        [
            "#[snapfit::static_dispatch] goes on a trait, and this item is not one; put it on the trait the container dispatches"
        ]
    );
    assert!(compact(&tokens).ends_with("pubstructDatabase;"));
}

#[test]
fn tokens_that_are_no_item_are_syn_s_own_error() {
    let tokens = static_dispatch_attribute(quote!(), quote!(not an item));

    assert!(compact(&tokens).contains("compile_error"));
}
