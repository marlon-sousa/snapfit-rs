use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::{compact, plan_for};

#[test]
fn test_sync_container_gets_new_default_and_container_impls() {
    let (plan, syntax) = plan_for(parse_quote! {
        struct App {
            #[singleton(deps = [config])]
            db: Arc<Database>,
            #[transient(deps = [config])]
            cart: Cart,
            #[singleton]
            config: Arc<Config>,
        }
    });

    let code = compact(emit_init_impl(&plan, &syntax));

    assert!(code.contains(
        "pubfnnew()->Self{letconfig:Arc<Config>=::std::sync::Arc::new(<Config>::new());letdb:Arc<Database>=::std::sync::Arc::new(<Database>::new(config.clone()));Self{config,db,}}"
    ));
    assert!(code.contains("impl::std::default::DefaultforApp"));
    assert!(code.contains("impl::snapfit::ContainerforApp"));
    assert!(!code.contains("letcart"));
}

#[test]
fn test_async_container_gets_init_returning_result() {
    let (plan, syntax) = plan_for(parse_quote! {
        struct App {
            #[singleton(async, constructor = "connect")]
            db: Arc<Database>,
        }
    });

    let code = compact(emit_init_impl(&plan, &syntax));

    assert!(code.contains("pubasyncfninit()->::std::result::Result<Self,"));
    assert!(code.contains(
        "::std::result::Result::Ok({letdb:Arc<Database>=::std::sync::Arc::new(<Database>::connect().await?);Self{db,}})"
    ));
    assert!(!code.contains("Default"));
}
