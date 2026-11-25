use quote::quote;
use syn::parse_quote;

use crate::adapters::outbound::dispatch_impls::emit_dispatch_impls;
use crate::adapters::read_dispatch_attribute;
use crate::test_support::compact;
use crate::use_cases::plan_dispatch;

fn emitted(item: syn::Item) -> proc_macro2::TokenStream {
    let plan = read_dispatch_attribute(&quote!(), &item)
        .and_then(plan_dispatch)
        .unwrap();
    emit_dispatch_impls(&plan)
}

#[test]
fn each_impl_assumes_every_supertrait_but_sized_relaxed_and_lifetimes() {
    let code = compact(emitted(parse_quote!(
        trait Database<'a>: Named + ?Sized + Send + 'a {
            fn find(&self) -> u32;
        }
    )));

    assert!(code.contains(&compact(quote!(
        where
            __SnapfitFirst: Database<'a>,
            __SnapfitRest: Database<'a>,
            ::snapfit::__private::Either<__SnapfitFirst, __SnapfitRest>: Named,
            ::snapfit::__private::Either<__SnapfitFirst, __SnapfitRest>: Send,
    ))));
    assert!(code.contains(&compact(quote!(
        where
            __SnapfitLast: Database<'a>,
            ::snapfit::__private::Only<__SnapfitLast>: Named,
            ::snapfit::__private::Only<__SnapfitLast>: Send,
    ))));
    assert!(!code.contains("?Sized"));
}

#[test]
fn each_supertrait_is_checked_for_the_last_implementation_with_the_trait_s_own_parameters() {
    let code = compact(emitted(parse_quote!(
        trait Store<T>: Named<T>
        where
            T: Copy,
        {
            fn find(&self) -> T;
        }
    )));

    assert!(code.contains(&compact(quote! {
        trait __SnapfitSupertrait<T> where T: Copy, {
            fn __snapfit_holds() {}
        }
        #[diagnostic::do_not_recommend]
        impl<T, __SnapfitHolder: ?::core::marker::Sized + Named<T> > __SnapfitSupertrait<T>
            for __SnapfitHolder where T: Copy, {}
        #[allow(dead_code)]
        fn __snapfit_supertrait<T, __SnapfitLast: Store<T> >() where T: Copy, {
            <::snapfit::__private::Only<__SnapfitLast> as __SnapfitSupertrait<T> >::__snapfit_holds();
        }
    })));
}

#[test]
fn a_supertrait_that_cannot_be_dispatched_is_reported_in_words_naming_it_and_the_trait() {
    let code = emitted(parse_quote!(
        trait Shown: std::fmt::Display {
            fn shown(&self) -> String;
        }
    ))
    .to_string();

    assert!(code.contains(
        "message = \"snapfit cannot dispatch the supertrait `Display` of `Shown` statically, because it is not marked #[snapfit::static_dispatch] or it is a trait of another crate\""
    ));
    assert!(code.contains(
        "note = \"mark `Display` with #[snapfit::static_dispatch] where it is defined; a supertrait of another crate, other than Send, Sync, Sized, Clone and Debug, cannot be dispatched statically, so dispatch `Shown` dynamically instead, through Arc<dyn Shown>\""
    ));
}
