use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::plan_for;
use crate::test_support::compact;

#[test]
fn a_documented_getter_clones_a_singleton_and_builds_a_transient_fresh() {
    let plan = plan_for(parse_quote! {
        struct App {
            #[singleton]
            config: std::sync::Arc<Config>,
            #[transient(deps = [config])]
            cart: Cart,
            #[singleton]
            r#type: Kind,
        }
    });

    let code = compact(emit_getters_impl(&plan));

    assert!(code.contains("pubfnget_config(&self)->std::sync::Arc<Config>{self.config.clone()}"));
    assert!(code.contains("pubfnget_cart(&self)->Cart{<Cart>::new(self.config.clone())}"));
    assert!(code.contains("pubfnget_type(&self)->Kind{self.r#type.clone()}"));
    assert!(code.contains(&compact(quote! {
        #[doc = "Returns a clone of `config`, built once with the container."]
        pub fn get_config
    })));
    assert!(code.contains(&compact(quote! {
        #[doc = "Builds a new `cart` for this call."]
        pub fn get_cart
    })));
    assert!(code.contains(&compact(quote! {
        #[doc = "Returns a clone of `type`, built once with the container."]
        pub fn get_type
    })));
}

#[test]
fn only_the_getter_of_a_dispatched_field_allows_a_more_private_implementation() {
    let plan = plan_for(parse_quote! {
        #[static_dispatch_for(Database as Db)]
        pub struct App {
            #[singleton(implementation = MySql)]
            db: Arc<Db>,
            #[singleton(implementation = MySql)]
            dynamic: Arc<dyn Database>,
        }
    });

    let code = compact(emit_getters_impl(&plan));

    assert!(code.contains("#[allow(private_interfaces)]pubfnget_db(&self)->Arc<Db>"));
    assert!(code.contains(&compact(quote! {
        #[doc = "Returns a clone of `dynamic`, built once with the container."]
        pub fn get_dynamic(&self) -> Arc<dyn Database>
    })));
}
