//! Tests: the value of a field marked `fallible = true`, wrapped into a `BuildError`.

use syn::parse_quote;

use super::{ValueSource, built};

#[test]
fn every_way_of_building_a_fallible_field_is_wrapped_with_the_names_rust_reads() {
    let item: syn::ItemStruct = parse_quote! {
        struct r#App {
            #[singleton]
            config: Config,
            #[singleton(deps = [config], fallible = true)]
            r#db: Arc<Database>,
            #[singleton(deps = [config], fallible = true, build = builder)]
            cache: Cache,
            #[singleton(deps = [config], fallible = true, build = factory)]
            gateway: Arc<dyn Gateway>,
        }
    };
    let wrapped = |built: &str, field: &str| {
        format!(
            "match{{let__snapfit_built={built};::snapfit::__private::BuildResult::into_build_result(__snapfit_built,\"App\",\"{field}\")}}{{::std::result::Result::Ok(__snapfit_value)=>__snapfit_value,::std::result::Result::Err(__snapfit_error)=>{{return::std::result::Result::Err(::std::convert::From::from(__snapfit_error))}}}}"
        )
    };

    let cases = [
        (
            "db",
            format!(
                "::std::sync::Arc::new({})",
                wrapped(
                    "<Database>::new(<Configas::std::clone::Clone>::clone(&__snapfit_config))",
                    "db"
                )
            ),
        ),
        (
            "cache",
            wrapped(
                "<Cache>::builder().config(<Configas::std::clone::Clone>::clone(&__snapfit_config)).build()",
                "cache",
            ),
        ),
        (
            "gateway",
            wrapped(
                "<SelfasAppFactories>::build_gateway(AppGatewayInputs{config:<Configas::std::clone::Clone>::clone(&__snapfit_config)})",
                "gateway",
            ),
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
