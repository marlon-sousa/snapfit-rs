use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::plan_for;
use crate::test_support::compact;

fn shop() -> syn::ItemStruct {
    parse_quote! {
        #[static_dispatch_for(Database as Db, extra = [Sqlite])]
        #[static_dispatch_for(Store<Order> as Orders)]
        #[extend_static_dispatch(Db, [Mock, MySql])]
        pub(crate) struct Shop {
            #[singleton(implementation = MySql)]
            orders_db: Arc<Db>,
            #[singleton(implementation = Cache<Order>)]
            orders: Arc<Orders>,
        }
    }
}

#[test]
fn each_alias_nests_its_implementations_once_with_the_container_s_visibility() {
    let plan = plan_for(shop());

    let aliases = compact(emit_aliases(&plan.dispatched_types, &plan.syntax.vis));

    assert_eq!(
        aliases,
        compact(quote! {
            #[allow(private_interfaces)]
            pub(crate) type Db = ::snapfit::__private::Either<
                MySql,
                ::snapfit::__private::Either<Sqlite, ::snapfit::__private::Only<Mock>>
            >;
            #[allow(private_interfaces)]
            pub(crate) type Orders = ::snapfit::__private::Only<Cache<Order> >;
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
