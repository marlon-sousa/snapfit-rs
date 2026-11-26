use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::{plan_for, step};
use crate::test_support::compact;

mod dispatched;
mod fallible;
mod implementation;

fn built(item: syn::ItemStruct, name: &str, source: ValueSource) -> String {
    let plan = plan_for(item);
    compact(construct(
        step(&plan, name),
        Site {
            container: &plan.syntax,
            source,
        },
    ))
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
        "::std::sync::Arc::new(<Mutex<Counter>>::new(<Counteras::std::clone::Clone>::clone(&__snapfit_counter)))"
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
        "::std::sync::Arc::new(<Orders>::new(<Checkout>::new(<Pricing>::new(<Arc<Config>as::std::clone::Clone>::clone(&__snapfit_config))),<Arc<Config>as::std::clone::Clone>::clone(&__snapfit_config)))"
    );
    assert_eq!(
        built(item, "checkout", ValueSource::SelfFields),
        "<Checkout>::new(<Pricing>::new(<Arc<Config>as::std::clone::Clone>::clone(&self.config)))"
    );
}

#[test]
fn an_async_constructor_is_awaited() {
    let item: syn::ItemStruct = parse_quote! {
        struct App {
            #[singleton]
            config: Arc<Config>,
            #[singleton(deps = [config], async, build = constructor(connect))]
            db: Arc<Database>,
        }
    };

    assert_eq!(
        built(item, "db", ValueSource::Locals),
        "::std::sync::Arc::new(<Database>::connect(<Arc<Config>as::std::clone::Clone>::clone(&__snapfit_config)).await?)"
    );
}

#[test]
fn a_factory_is_called_through_its_trait_with_its_inputs_by_name() {
    let item: syn::ItemStruct = parse_quote! {
        struct Shop {
            #[singleton]
            read_db: Arc<Database>,
            #[transient]
            clock: Clock,
            #[singleton(deps = [clock, read_db], build = factory)]
            orders: Arc<dyn OrderStore>,
        }
    };

    assert_eq!(
        built(item.clone(), "orders", ValueSource::Locals),
        "<SelfasShopFactories>::build_orders(ShopOrdersInputs{clock:<Clock>::new(),read_db:<Arc<Database>as::std::clone::Clone>::clone(&__snapfit_read_db)})"
    );
    assert_eq!(
        built(item, "orders", ValueSource::SelfFields),
        "<SelfasShopFactories>::build_orders(ShopOrdersInputs{clock:<Clock>::new(),read_db:<Arc<Database>as::std::clone::Clone>::clone(&self.read_db)})"
    );
}

#[test]
fn default_calls_the_default_trait_and_wraps_an_arc() {
    let item: syn::ItemStruct = parse_quote! {
        struct Shop {
            #[singleton(build = default)]
            rates: Arc<TaxRates>,
            #[singleton(build = default)]
            clock: Clock,
        }
    };

    assert_eq!(
        built(item.clone(), "rates", ValueSource::Locals),
        "::std::sync::Arc::new(<TaxRatesas::std::default::Default>::default())"
    );
    assert_eq!(
        built(item, "clock", ValueSource::Locals),
        "<Clockas::std::default::Default>::default()"
    );
}

#[test]
fn a_builder_passes_each_dependency_to_its_setter_and_finishes_the_chain() {
    let item: syn::ItemStruct = parse_quote! {
        struct App {
            #[singleton]
            folder_files: Arc<FolderFiles>,
            #[singleton]
            memory_files: Arc<MemoryFiles>,
            #[singleton(deps = [memory = memory_files, disk = folder_files], build = builder)]
            writer: Arc<SiteWriter>,
            #[singleton(deps = [folder_files], build = builder(start = make, finish = done))]
            reader: Reader,
        }
    };

    assert_eq!(
        built(item.clone(), "writer", ValueSource::Locals),
        "::std::sync::Arc::new(<SiteWriter>::builder().memory(<Arc<MemoryFiles>as::std::clone::Clone>::clone(&__snapfit_memory_files)).disk(<Arc<FolderFiles>as::std::clone::Clone>::clone(&__snapfit_folder_files)).build())"
    );
    assert_eq!(
        built(item, "reader", ValueSource::SelfFields),
        "<Reader>::make().folder_files(<Arc<FolderFiles>as::std::clone::Clone>::clone(&self.folder_files)).done()"
    );
}
