use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::{plan_for, step};
use crate::test_support::compact;

fn built(item: syn::ItemStruct, name: &str, source: ValueSource) -> String {
    let plan = plan_for(item);
    compact(construct(step(&plan, name), source))
}

#[test]
fn a_generic_or_qualified_type_is_constructed_through_a_qualified_path() {
    let item: syn::ItemStruct = parse_quote! {
        struct App {
            #[singleton]
            names: Vec<String>,
            #[singleton]
            config: ::std::sync::Arc<Config>,
            #[singleton]
            counter: Counter,
            #[singleton(deps = [counter])]
            shared: Arc<Mutex<Counter>>,
        }
    };

    assert_eq!(
        built(item.clone(), "names", ValueSource::Locals),
        "<Vec<String>>::new()"
    );
    assert_eq!(
        built(item.clone(), "config", ValueSource::Locals),
        "::std::sync::Arc::new(<Config>::new())"
    );
    assert_eq!(
        built(item, "shared", ValueSource::Locals),
        "::std::sync::Arc::new(<Mutex<Counter>>::new(__snapfit_counter.clone()))"
    );
}

#[test]
fn a_nested_transient_is_built_fresh_for_each_argument() {
    let item: syn::ItemStruct = parse_quote! {
        struct App {
            #[singleton]
            config: Arc<Config>,
            #[singleton(deps = [checkout, config])]
            orders: Arc<Orders>,
            #[transient(deps = [pricing])]
            checkout: Checkout,
            #[transient(deps = [config])]
            pricing: Pricing,
        }
    };

    assert_eq!(
        built(item.clone(), "orders", ValueSource::Locals),
        "::std::sync::Arc::new(<Orders>::new(<Checkout>::new(<Pricing>::new(__snapfit_config.clone())),__snapfit_config.clone()))"
    );
    assert_eq!(
        built(item, "checkout", ValueSource::SelfFields),
        "<Checkout>::new(<Pricing>::new(self.config.clone()))"
    );
}

#[test]
fn an_async_constructor_is_awaited_and_a_builder_is_called_on_self() {
    let item: syn::ItemStruct = parse_quote! {
        struct App {
            #[singleton]
            config: Arc<Config>,
            #[singleton(deps = [config], async, constructor = connect)]
            db: Arc<Database>,
            #[singleton(deps = [config, db], builder = build_auth)]
            auth: Arc<AuthService>,
        }
    };

    assert_eq!(
        built(item.clone(), "db", ValueSource::Locals),
        "::std::sync::Arc::new(<Database>::connect(__snapfit_config.clone()).await?)"
    );
    assert_eq!(
        built(item, "auth", ValueSource::Locals),
        "Self::build_auth(__snapfit_config.clone(),__snapfit_db.clone())"
    );
}
