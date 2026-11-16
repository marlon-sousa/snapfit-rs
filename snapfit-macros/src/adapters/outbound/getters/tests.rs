use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::{compact, plan_for};

#[test]
fn test_getters_clone_singletons_and_build_transients() {
    let (plan, syntax) = plan_for(parse_quote! {
        struct App {
            #[singleton]
            config: std::sync::Arc<Config>,
            #[transient(deps = [config])]
            cart: Cart,
            #[singleton]
            r#type: Kind,
        }
    });

    let code = compact(emit_getters_impl(&plan, &syntax));

    assert!(code.contains("pubfnget_config(&self)->std::sync::Arc<Config>{self.config.clone()}"));
    assert!(code.contains("pubfnget_cart(&self)->Cart{<Cart>::new(self.config.clone())}"));
    assert!(code.contains("pubfnget_type(&self)->Kind{self.r#type.clone()}"));
}
