use syn::parse_quote;

use super::test_support::plan_for;
use super::*;

#[test]
fn a_container_is_emitted_as_a_struct_and_its_impls_that_parse() {
    let plan = plan_for(parse_quote! {
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

    let tokens = emit_container(&plan);
    let file: syn::File = syn::parse2(tokens).expect("emitted tokens must be valid Rust");

    // struct, inherent new, Default, Container, getters
    assert_eq!(file.items.len(), 5);
    let syn::Item::Struct(item) = &file.items[0] else {
        panic!("the struct comes first");
    };
    assert_eq!(item.attrs.len(), 1);
    assert_eq!(item.fields.len(), 2);
}
