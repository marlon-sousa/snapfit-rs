use syn::parse_quote;

use super::built;
use crate::adapters::outbound::expr::ValueSource;

#[test]
fn a_dispatched_field_wraps_the_implementation_it_names_in_its_variant_then_in_its_arc() {
    let item: syn::ItemStruct = parse_quote! {
        #[static_dispatch_for(Database as Db)]
        struct App {
            #[singleton]
            config: Arc<Config>,
            #[singleton(implementation = MySql)]
            plain: Arc<Db>,
            #[singleton(deps = [config], implementation = Postgres, build = constructor(connect))]
            connected: std::sync::Arc<Db>,
            #[singleton(implementation = Memory, build = default)]
            defaulted: Arc<Db>,
            #[singleton(deps = [config], implementation = Cache<Order>, build = builder)]
            built: Arc<Db>,
        }
    };

    let cases = [
        ("plain", "::std::sync::Arc::new(<Db>::of(<MySql>::new()))"),
        (
            "connected",
            "::std::sync::Arc::new(<Db>::of(<Postgres>::connect(__snapfit_config.clone())))",
        ),
        (
            "defaulted",
            "::std::sync::Arc::new(<Db>::of(<Memoryas::std::default::Default>::default()))",
        ),
        (
            "built",
            "::std::sync::Arc::new(<Db>::of(<Cache<Order>>::builder().config(__snapfit_config.clone()).build()))",
        ),
    ];
    for (field, expected) in cases {
        assert_eq!(
            built(item.clone(), field, ValueSource::Locals),
            expected,
            "{field}"
        );
    }
}

#[test]
fn a_fallible_dispatched_field_unwraps_the_implementation_before_wrapping_it() {
    let item: syn::ItemStruct = parse_quote! {
        #[static_dispatch_for(Database as Db)]
        struct App {
            #[singleton(implementation = MySql, fallible = true)]
            db: Arc<Db>,
        }
    };

    let code = built(item, "db", ValueSource::Locals);

    assert!(
        code.starts_with("::std::sync::Arc::new(<Db>::of("),
        "{code}"
    );
    assert!(code.contains("<MySql>::new()"), "{code}");
}
