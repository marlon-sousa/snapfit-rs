use syn::parse_quote;

use super::test_support::plan_for;
use super::*;
use crate::ports::CodeEmitterPort;

#[test]
fn test_emit_container_produces_parseable_struct_and_impls() {
    let (plan, syntax) = plan_for(parse_quote! {
        #[derive(Clone)]
        pub struct AppContainer {
            #[singleton]
            config: Arc<Config>,
            #[singleton(deps = [config])]
            db: Arc<Database>,
            #[transient(deps = [config, db])]
            user_controller: UserController,
        }
    });

    let tokens = QuoteEmitterAdapter::new().emit_container(&plan, &syntax);
    let file: syn::File = syn::parse2(tokens).expect("emitted tokens must be valid Rust");

    // struct, inherent new, Default, Container, getters
    assert_eq!(file.items.len(), 5);
    let syn::Item::Struct(item) = &file.items[0] else {
        panic!("the struct comes first");
    };
    assert_eq!(item.attrs.len(), 1);
    assert_eq!(item.fields.len(), 2);
}

#[test]
fn test_emit_derive_impls_does_not_repeat_the_struct() {
    let (plan, syntax) = plan_for(parse_quote! {
        pub struct AppContainer {
            #[singleton]
            config: Arc<Config>,
        }
    });

    let tokens = QuoteEmitterAdapter::new().emit_derive_impls(&plan, &syntax);
    let file: syn::File = syn::parse2(tokens).expect("emitted tokens must be valid Rust");

    assert_eq!(file.items.len(), 4);
    assert!(
        file.items
            .iter()
            .all(|item| !matches!(item, syn::Item::Struct(_)))
    );
}
