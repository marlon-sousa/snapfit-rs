use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::plan_for;
use crate::test_support::compact;

#[test]
fn the_struct_keeps_the_users_attributes_visibility_and_field_order() {
    let plan = plan_for(parse_quote! {
        /// The shop.
        #[derive(Clone, Debug)]
        pub(crate) struct Shop {
            /// Checkout carts.
            #[transient(deps = [config])]
            cart: Cart,
            #[singleton(deps = [config])]
            #[allow(dead_code)]
            pub db: Arc<Database>,
            /// Loaded first.
            #[singleton]
            config: Arc<Config>,
        }
    });

    let code = compact(emit_struct_definition(&plan));

    assert_eq!(
        code,
        "#[doc=r\"Theshop.\"]#[derive(Clone,Debug)]pub(crate)structShop{#[allow(dead_code)]#[allow(dead_code)]pubdb:Arc<Database>,#[doc=r\"Loadedfirst.\"]#[allow(dead_code)]config:Arc<Config>,}"
    );
}

#[test]
fn a_private_struct_stays_private() {
    let plan = plan_for(parse_quote! {
        struct Hidden {
            #[singleton]
            config: Arc<Config>,
        }
    });

    assert_eq!(
        compact(emit_struct_definition(&plan)),
        "structHidden{#[allow(dead_code)]config:Arc<Config>,}"
    );
}
