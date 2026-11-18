use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::plan_for;
use crate::test_support::compact;

#[test]
fn a_sync_container_gets_new_default_and_the_container_impl() {
    let plan = plan_for(parse_quote! {
        struct App {
            #[singleton(deps = [config])]
            db: Arc<Database>,
            #[transient(deps = [config])]
            cart: Cart,
            #[singleton]
            config: Arc<Config>,
        }
    });

    let code = compact(emit_init_impl(&plan));

    assert!(code.contains(
        "pubfnnew()->Self{let__snapfit_config:Arc<Config>=::std::sync::Arc::new(<Config>::new());let__snapfit_db:Arc<Database>=::std::sync::Arc::new(<Database>::new(__snapfit_config.clone()));Self{config:__snapfit_config,db:__snapfit_db,}}"
    ));
    assert!(code.contains("impl::std::default::DefaultforApp"));
    assert!(code.contains("impl::snapfit::ContainerforApp"));
    assert!(!code.contains("letcart"));
}

#[test]
fn an_async_container_gets_init_returning_a_result() {
    let plan = plan_for(parse_quote! {
        struct App {
            #[singleton(async, build = constructor(connect))]
            db: Arc<Database>,
        }
    });

    let code = compact(emit_init_impl(&plan));

    assert!(code.contains("pubasyncfninit()->::std::result::Result<Self,"));
    assert!(code.contains(
        "::std::result::Result::Ok({let__snapfit_db:Arc<Database>=::std::sync::Arc::new(<Database>::connect().await?);Self{db:__snapfit_db,}})"
    ));
    assert!(!code.contains("Default"));
}
