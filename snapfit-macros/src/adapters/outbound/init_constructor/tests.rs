use syn::parse_quote;

use super::*;
use crate::adapters::outbound::test_support::plan_for;
use crate::test_support::compact;

#[test]
fn a_sync_container_gets_a_documented_new_default_and_the_container_impl() {
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
    assert!(code.contains(&compact(quote! {
        #[doc = "Builds `App` with each of its singletons, in the order their dependencies need."]
        pub fn new()
    })));
    assert!(code.contains("impl::std::default::DefaultforApp"));
    assert!(code.contains("impl::snapfit::ContainerforApp"));
    assert!(!code.contains("letcart"));
}

#[test]
fn an_async_container_gets_a_documented_init_returning_a_result() {
    let plan = plan_for(parse_quote! {
        struct App {
            #[singleton(async, build = constructor(connect))]
            db: Arc<Database>,
        }
    });

    let code = compact(emit_init_impl(&plan));

    assert!(code.contains("pubasyncfninit()->::std::result::Result<Self,"));
    assert!(code.contains(&compact(quote! {
        #[doc = "Builds `App` with each of its singletons, awaiting those built asynchronously, or returns the first error."]
        pub async fn init()
    })));
    assert!(code.contains(
        "::std::result::Result::Ok({let__snapfit_db:Arc<Database>=::std::sync::Arc::new(<Database>::connect().await?);Self{db:__snapfit_db,}})"
    ));
    assert!(!code.contains("Default"));
}

#[test]
fn a_container_with_a_fallible_field_gets_a_documented_try_new_and_the_fallible_container_impl_only()
 {
    let plan = plan_for(parse_quote! {
        struct App {
            #[singleton(fallible = true, build = constructor(load))]
            config: Arc<Config>,
            #[singleton(deps = [config])]
            db: Arc<Database>,
        }
    });

    let code = compact(emit_init_impl(&plan));

    assert!(code.contains(&compact(quote! {
        #[doc = "Builds `App` with each of its singletons, or returns the `snapfit::BuildError` of the first field that fails."]
        pub fn try_new() -> ::std::result::Result<Self, ::snapfit::BuildError>
    })));
    assert!(code.contains(
        "::std::result::Result::Ok({let__snapfit_config:Arc<Config>=::std::sync::Arc::new(match{let__snapfit_built=<Config>::load();::snapfit::__private::BuildResult::into_build_result(__snapfit_built,\"App\",\"config\")}{::std::result::Result::Ok(__snapfit_value)=>__snapfit_value,::std::result::Result::Err(__snapfit_error)=>{return::std::result::Result::Err(::std::convert::From::from(__snapfit_error))}});let__snapfit_db:Arc<Database>=::std::sync::Arc::new(<Database>::new(__snapfit_config.clone()));Self{config:__snapfit_config,db:__snapfit_db,}})"
    ));
    assert!(code.contains("impl::snapfit::FallibleContainerforApp"));
    assert!(!code.contains("pubfnnew()"));
    assert!(!code.contains("Default"));
    assert!(!code.contains("::snapfit::ContainerforApp"));
}

#[test]
fn an_async_container_wraps_a_fallible_field_after_awaiting_it() {
    let plan = plan_for(parse_quote! {
        struct App {
            #[singleton(async, fallible = true, build = constructor(connect))]
            db: Database,
        }
    });

    let code = compact(emit_init_impl(&plan));

    assert!(code.contains("pubasyncfninit()"));
    assert!(code.contains(
        "match{let__snapfit_built=<Database>::connect().await;::snapfit::__private::BuildResult::into_build_result(__snapfit_built,\"App\",\"db\")}{::std::result::Result::Ok(__snapfit_value)=>__snapfit_value,::std::result::Result::Err(__snapfit_error)=>{return::std::result::Result::Err(::std::convert::From::from(__snapfit_error))}}"
    ));
    assert!(!code.contains("try_new"));
}
