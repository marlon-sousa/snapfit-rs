use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::plan_for;
use crate::test_support::compact;

#[test]
fn a_getter_clones_a_singleton_and_builds_a_transient_fresh() {
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
}
