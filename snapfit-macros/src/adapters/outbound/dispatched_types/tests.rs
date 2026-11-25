use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::plan_for;
use crate::test_support::compact;

fn shop() -> syn::ItemStruct {
    parse_quote! {
        #[static_dispatch_for(Database as Db)]
        #[static_dispatch_for(Store<Order> as Orders)]
        #[extend_static_dispatch(Db, [Mock, MySql])]
        pub(crate) struct Shop {
            #[singleton(implementation = MySql)]
            orders_db: Arc<Db>,
            #[singleton(implementation = Cache<Order>)]
            orders: Arc<Orders>,
            #[singleton(build = factory, implementations = [Sqlite, MySql])]
            reports_db: Arc<Db>,
        }
    }
}

#[test]
fn each_alias_is_documented_and_nests_its_implementations_once_with_the_container_s_visibility() {
    let plan = plan_for(shop());
    let mut types = plan
        .dispatched_types
        .into_iter()
        .map(DispatchedTypeFallback::Planned);
    let mut fallbacks: Vec<_> = types.by_ref().collect();
    // A type that holds no implementation stands for unit.
    let unplanned: DispatchSyntax = DispatchSyntax {
        trait_path: parse_quote!(Clock),
        name: parse_quote!(Clocks),
    };
    fallbacks.push(DispatchedTypeFallback::Unplanned(unplanned));

    let aliases = compact(emit_fallback_aliases(&fallbacks, &plan.syntax.vis));

    assert_eq!(
        aliases,
        compact(quote! {
            #[doc = "One of the implementations of `Database` that the container dispatches statically; `Db::of(value)` wraps one."]
            #[allow(private_interfaces)]
            pub(crate) type Db = ::snapfit::__private::Either<
                MySql,
                ::snapfit::__private::Either<Sqlite, ::snapfit::__private::Only<Mock>>
            >;
            #[doc = "One of the implementations of `Store` that the container dispatches statically; `Orders::of(value)` wraps one."]
            #[allow(private_interfaces)]
            pub(crate) type Orders = ::snapfit::__private::Only<Cache<Order> >;
            pub(crate) type Clocks = ();
        })
    );
}

#[test]
fn the_dispatched_type_and_each_implementation_are_checked_against_the_trait() {
    let plan = plan_for(shop());

    let code = compact(emit_dispatched_types(
        &plan.dispatched_types,
        &plan.syntax.vis,
    ));

    assert!(code.contains(&compact(quote! {
        const _: () = {
            fn __snapfit_implements<__SnapfitDispatched: ?::core::marker::Sized + Database>() {}
            let _ = __snapfit_implements::<Db>;
            let _ = __snapfit_implements::<MySql>;
            let _ = __snapfit_implements::<Sqlite>;
            let _ = __snapfit_implements::<Mock>;
        };
    })));
    assert!(code.contains(&compact(quote! {
        fn __snapfit_implements<__SnapfitDispatched: ?::core::marker::Sized + Store<Order> >() {}
        let _ = __snapfit_implements::<Orders>;
        let _ = __snapfit_implements::<Cache<Order> >;
    })));
}

#[test]
fn each_other_spelling_of_an_implementation_is_checked_to_be_the_type_held() {
    let plan = plan_for(parse_quote! {
        #[static_dispatch_for(Database as Db)]
        #[extend_static_dispatch(Db, [self::MySql, Mock])]
        struct Shop {
            #[singleton(implementation = MySql)]
            orders_db: Arc<Db>,
        }
    });

    let code = compact(emit_dispatched_types(
        &plan.dispatched_types,
        &plan.syntax.vis,
    ));

    assert!(code.contains(&compact(quote! {
        let _ = __snapfit_implements::<Mock>;
        let _ = ::snapfit::__private::same_type::<self::MySql, MySql>;
    })));
    assert!(!code.contains("__snapfit_implements::<self::MySql>"));
}

#[test]
fn an_alias_s_doc_names_a_raw_trait_and_alias_without_r_hash() {
    let plan = plan_for(parse_quote! {
        #[static_dispatch_for(r#Database as r#Db)]
        struct Shop {
            #[singleton(implementation = MySql)]
            orders_db: Arc<r#Db>,
        }
    });

    let code = compact(emit_dispatched_types(
        &plan.dispatched_types,
        &plan.syntax.vis,
    ));

    assert!(code.contains(&compact(quote! {
        #[doc = "One of the implementations of `Database` that the container dispatches statically; `Db::of(value)` wraps one."]
    })));
}
