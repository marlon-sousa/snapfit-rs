use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::plan_for;
use crate::test_support::compact;

#[test]
fn no_factory_field_generates_nothing() {
    let plan = plan_for(parse_quote! {
        pub struct Shop {
            #[singleton]
            config: Arc<Config>,
        }
    });

    assert!(emit_factories(&plan).is_empty());
}

#[test]
fn the_trait_has_one_build_method_per_factory_taking_its_inputs_struct() {
    let plan = plan_for(parse_quote! {
        pub(crate) struct Shop {
            #[singleton]
            config: Arc<Config>,
            #[singleton(deps = [config], build = factory)]
            read_db: Arc<Database>,
            #[singleton(deps = [config])]
            write_db: Arc<Database>,
            #[singleton(deps = [read_db, write_db], build = factory)]
            order_store: Arc<dyn OrderStore>,
        }
    });

    let code = compact(emit_factories(&plan));

    assert!(code.contains("pub(crate)traitShopFactories{"), "{code}");
    assert!(code.contains("fnbuild_read_db(inputs:ShopReadDbInputs)->Arc<Database>;"));
    assert!(code.contains("fnbuild_order_store(inputs:ShopOrderStoreInputs)->Arc<dynOrderStore>;"));
    assert!(code.contains("pub(crate)structShopReadDbInputs{"));
    assert!(code.contains("pubconfig:Arc<Config>,}"));
    assert!(code.contains("pub(crate)structShopOrderStoreInputs{"));
    assert!(code.contains("pubread_db:Arc<Database>,"));
    assert!(code.contains("pubwrite_db:Arc<Database>,}"));
    assert!(!code.contains("build_write_db"));
    assert!(!code.contains("build_config"));
}

#[test]
fn a_factory_without_deps_still_takes_an_empty_inputs_struct() {
    let plan = plan_for(parse_quote! {
        struct Shop {
            #[singleton(build = factory)]
            clock: Clock,
        }
    });

    let code = compact(emit_factories(&plan));

    assert!(code.contains("fnbuild_clock(inputs:ShopClockInputs)->Clock;"));
    assert!(code.contains("structShopClockInputs{}"));
}

#[test]
fn generated_names_drop_the_raw_prefix_and_write_the_field_in_pascal_case() {
    let cases = [
        (
            quote::format_ident!("Shop"),
            quote::format_ident!("order_store"),
            "ShopOrderStoreInputs",
            "build_order_store",
        ),
        (
            quote::format_ident!("Shop"),
            syn::Ident::new_raw("type", proc_macro2::Span::call_site()),
            "ShopTypeInputs",
            "build_type",
        ),
        (
            quote::format_ident!("Shop"),
            quote::format_ident!("db2_pool"),
            "ShopDb2PoolInputs",
            "build_db2_pool",
        ),
    ];
    for (container, field, inputs, method) in cases {
        assert_eq!(inputs_ident(&container, &field).to_string(), inputs);
        assert_eq!(method_ident(&field).to_string(), method);
    }
    assert_eq!(
        trait_ident(&quote::format_ident!("Shop")).to_string(),
        "ShopFactories"
    );
}

#[test]
fn a_fallible_factory_returns_a_result_of_its_field_s_type_with_a_boxed_error() {
    let plan = plan_for(parse_quote! {
        pub struct Shop {
            #[singleton(fallible = true, build = factory)]
            gateway: Arc<dyn Gateway>,
        }
    });

    let code = compact(emit_factories(&plan));

    assert!(code.contains(
        "fnbuild_gateway(inputs:ShopGatewayInputs)->::std::result::Result<Arc<dynGateway>,::std::boxed::Box<dyn::std::error::Error+::std::marker::Send+::std::marker::Sync>,>;"
    ));
}

#[test]
fn only_an_input_of_a_dispatched_type_allows_a_more_private_implementation() {
    let plan = plan_for(parse_quote! {
        #[static_dispatch_for(Database as Db)]
        pub struct Shop {
            #[singleton]
            config: Arc<Config>,
            #[singleton(implementation = MySql)]
            db: Arc<Db>,
            #[singleton(deps = [config, db], build = factory)]
            orders: Arc<Orders>,
        }
    });

    let code = compact(emit_factories(&plan));

    assert!(code.contains("pubconfig:Arc<Config>,"), "{code}");
    assert!(
        !code.contains("#[allow(private_interfaces)]pubconfig"),
        "{code}"
    );
    assert!(
        code.contains("#[allow(private_interfaces)]pubdb:Arc<Db>,"),
        "{code}"
    );
}

#[test]
fn only_the_build_method_of_a_dispatched_field_allows_a_more_private_implementation() {
    let plan = plan_for(parse_quote! {
        #[static_dispatch_for(Database as Db, extra = [MySql])]
        pub struct Shop {
            #[singleton(build = factory)]
            shared: Arc<Db>,
            #[singleton(build = factory)]
            owned: Db,
            #[singleton(build = factory)]
            orders: Arc<Orders>,
        }
    });

    let code = compact(emit_factories(&plan));

    assert!(
        code.contains(
            "#[allow(private_interfaces)]fnbuild_shared(inputs:ShopSharedInputs)->Arc<Db>;"
        ),
        "{code}"
    );
    assert!(
        code.contains("#[allow(private_interfaces)]fnbuild_owned(inputs:ShopOwnedInputs)->Db;"),
        "{code}"
    );
    assert!(
        code.contains("fnbuild_orders(inputs:ShopOrdersInputs)->Arc<Orders>;"),
        "{code}"
    );
    assert!(
        !code.contains("#[allow(private_interfaces)]fnbuild_orders"),
        "{code}"
    );
}
