use quote::quote;
use syn::parse_quote;

use super::*;
use crate::adapters::read_dispatch_attribute;
use crate::test_support::compact;
use crate::use_cases::plan_dispatch;

/// The impls written for `item`, as compact text.
fn emitted(item: syn::Item) -> String {
    let plan = read_dispatch_attribute(&quote!(), &item)
        .and_then(plan_dispatch)
        .unwrap();
    compact(emit_dispatch_impls(&plan))
}

#[test]
fn either_forwards_each_method_to_the_variant_it_holds() {
    let code = emitted(parse_quote!(
        trait Database {
            fn find(&self, id: u32) -> Option<Order>;
        }
    ));

    assert!(code.contains(&compact(quote!(
        #[automatically_derived]
        #[doc(hidden)]
        #[allow(deprecated)]
        impl<__SnapfitFirst, __SnapfitRest> Database
            for ::snapfit::__private::Either<__SnapfitFirst, __SnapfitRest>
        where
            __SnapfitFirst: Database,
            __SnapfitRest: Database,
    ))));
    assert!(code.contains(&compact(quote!(
        ::snapfit::__private::Either::First(__snapfit_inner) =>
            <__SnapfitFirst as Database>::find(__snapfit_inner, id),
        ::snapfit::__private::Either::Rest(__snapfit_inner) =>
            <__SnapfitRest as Database>::find(__snapfit_inner, id),
    ))));
}

#[test]
fn only_forwards_each_method_to_the_last_implementation() {
    let code = emitted(parse_quote!(
        trait Database {
            fn find(&self, id: u32) -> Option<Order>;
        }
    ));

    assert!(code.contains(&compact(quote!(
        #[automatically_derived]
        #[doc(hidden)]
        #[allow(deprecated)]
        impl<__SnapfitLast> Database for ::snapfit::__private::Only<__SnapfitLast>
        where
            __SnapfitLast: Database,
    ))));
    assert!(code.contains(&compact(quote!(
        ::snapfit::__private::Only(__snapfit_inner) =>
            <__SnapfitLast as Database>::find(__snapfit_inner, id),
    ))));
}

#[test]
fn associated_types_come_from_the_first_implementation_and_the_rest_must_agree() {
    let code = emitted(parse_quote!(
        trait Database {
            type Row;
            fn rows(&self) -> Vec<Self::Row>;
        }
    ));

    assert!(code.contains(&compact(quote!(
        __SnapfitRest: Database<Row = <__SnapfitFirst as Database>::Row>,
    ))));
    assert!(code.contains(&compact(quote!(
        type Row = <__SnapfitFirst as Database>::Row;
    ))));
    assert!(code.contains(&compact(quote!(
        type Row = <__SnapfitLast as Database>::Row;
    ))));
}

#[test]
fn the_trait_s_parameters_and_where_clause_are_carried_without_their_defaults() {
    let code = emitted(parse_quote!(
        trait Store<'a, T: Clone = u32, const N: usize = 3>
        where
            T: 'a,
        {
            fn put(&self, item: &'a T);
        }
    ));

    assert!(code.contains(&compact(quote!(
        impl<'a, T: Clone, const N: usize, __SnapfitFirst, __SnapfitRest> Store<'a, T, N>
            for ::snapfit::__private::Either<__SnapfitFirst, __SnapfitRest>
        where
            T: 'a,
            __SnapfitFirst: Store<'a, T, N>,
            __SnapfitRest: Store<'a, T, N>,
    ))));
}

#[test]
fn a_method_returning_self_is_rebuilt_into_the_variant_it_came_from() {
    let code = emitted(parse_quote!(
        trait Database {
            fn renamed(self) -> Self;
        }
    ));

    assert!(code.contains(&compact(quote!(
        ::snapfit::__private::Either::Rest(__snapfit_inner) =>
            ::snapfit::__private::Either::Rest(<__SnapfitRest as Database>::renamed(__snapfit_inner)),
    ))));
}

#[test]
fn an_async_method_is_awaited_and_an_unsafe_one_called_in_an_unsafe_block() {
    let code = emitted(parse_quote!(
        trait Database {
            async fn ping(&self) -> bool;
            unsafe fn raw(&self) -> u8;
        }
    ));

    assert!(code.contains(&compact(quote!(
        <__SnapfitLast as Database>::ping(__snapfit_inner).await
    ))));
    assert!(code.contains(&compact(quote!(unsafe {
        <__SnapfitLast as Database>::raw(__snapfit_inner)
    }))));
}

#[test]
fn a_generic_method_passes_its_type_parameters_and_not_its_lifetimes() {
    let code = emitted(parse_quote!(
        trait Database {
            fn made<'a, T: Default, const N: usize>(&'a self) -> T;
        }
    ));

    assert!(
        code.contains(&compact(quote!(<__SnapfitLast as Database>::made::<T, N>(
            __snapfit_inner
        ))))
    );
}

#[test]
fn an_argument_written_as_a_pattern_or_mut_is_passed_on_by_a_plain_name() {
    let code = emitted(parse_quote!(
        trait Database {
            fn shown(mut self, (a, b): (u32, u32), mut count: u8) -> String {
                String::new()
            }
        }
    ));

    assert!(code.contains(&compact(quote!(
        fn shown(self, __snapfit_argument_1: (u32, u32), count: u8) -> String
    ))));
    assert!(
        code.contains(&compact(quote!(<__SnapfitLast as Database>::shown(
            __snapfit_inner,
            __snapfit_argument_1,
            count
        ))))
    );
}

#[test]
fn a_mut_reference_receiver_keeps_its_mut() {
    let code = emitted(parse_quote!(
        trait Database {
            fn store(&mut self, id: u32);
        }
    ));

    assert!(code.contains(&compact(quote!(fn store(&mut self, id: u32)))));
}

#[test]
fn a_cfg_on_an_item_is_kept_on_its_forwarding_item() {
    let code = emitted(parse_quote!(
        trait Database {
            #[cfg(test)]
            fn reset(&self);
        }
    ));

    assert!(code.contains(&compact(quote!(
        #[cfg(test)]
        #[inline]
        fn reset(&self)
    ))));
}
