use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::{compact, plan_for};
use crate::domain::DependencyName;

fn construct(item: syn::ItemStruct, name: &str, source: ValueSource) -> String {
    let (plan, syntax) = plan_for(item);
    let step = plan
        .step(&DependencyName::new(name).unwrap())
        .unwrap()
        .clone();
    compact(ExprBuilder::new(&plan, &syntax, source).construct(&step))
}

#[test]
fn test_arc_inner_type_recognises_only_std_arc_paths() {
    let arcs: [syn::Type; 4] = [
        parse_quote!(Arc<Config>),
        parse_quote!(std::sync::Arc<Config>),
        parse_quote!(::std::sync::Arc<Config>),
        parse_quote!(alloc::sync::Arc<Config>),
    ];
    arcs.iter().for_each(|ty| {
        let inner = arc_inner_type(ty).expect("an Arc path");
        assert_eq!(compact(quote!(#inner)), "Config");
    });

    let not_arcs: [syn::Type; 5] = [
        parse_quote!(Vec<Config>),
        parse_quote!(std::rc::Rc<Config>),
        parse_quote!(::Arc<Config>),
        parse_quote!(my::Arc<Config>),
        parse_quote!(Arc<Config, Global>),
    ];
    not_arcs
        .iter()
        .for_each(|ty| assert!(arc_inner_type(ty).is_none(), "{}", quote!(#ty)));
}

#[test]
fn test_construct_uses_qualified_paths_for_generic_and_qualified_types() {
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
        construct(item.clone(), "names", ValueSource::Locals),
        "<Vec<String>>::new()"
    );
    assert_eq!(
        construct(item.clone(), "config", ValueSource::Locals),
        "::std::sync::Arc::new(<Config>::new())"
    );
    assert_eq!(
        construct(item, "shared", ValueSource::Locals),
        "::std::sync::Arc::new(<Mutex<Counter>>::new(counter.clone()))"
    );
}

#[test]
fn test_construct_builds_nested_transients_fresh_for_each_argument() {
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
        construct(item.clone(), "orders", ValueSource::Locals),
        "::std::sync::Arc::new(<Orders>::new(<Checkout>::new(<Pricing>::new(config.clone())),config.clone()))"
    );
    assert_eq!(
        construct(item, "checkout", ValueSource::SelfFields),
        "<Checkout>::new(<Pricing>::new(self.config.clone()))"
    );
}

#[test]
fn test_construct_awaits_async_constructors_and_calls_builders_on_self() {
    let item: syn::ItemStruct = parse_quote! {
        struct App {
            #[singleton]
            config: Arc<Config>,
            #[singleton(deps = [config], async, constructor = "connect")]
            db: Arc<Database>,
            #[singleton(deps = [config, db], builder = "build_auth")]
            auth: Arc<AuthService>,
        }
    };

    assert_eq!(
        construct(item.clone(), "db", ValueSource::Locals),
        "::std::sync::Arc::new(<Database>::connect(config.clone()).await?)"
    );
    assert_eq!(
        construct(item, "auth", ValueSource::Locals),
        "Self::build_auth(config.clone(),db.clone())"
    );
}
