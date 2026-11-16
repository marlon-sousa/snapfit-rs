use quote::quote;
use syn::parse_quote;

use super::*;

#[test]
fn test_expand_container_attribute_with_and_without_args() {
    let item_struct: syn::ItemStruct = parse_quote! {
        pub struct AppContainer {
            #[singleton]
            config: Arc<Config>,

            #[singleton(deps = [config])]
            db: Arc<Database>,
        }
    };

    let tokens_no_args = expand_container_attribute(quote!(), &item_struct).unwrap();
    let code_no_args = tokens_no_args.to_string();
    assert!(code_no_args.contains("pub struct AppContainer"));
    assert!(code_no_args.contains("pub fn new () -> Self"));
    assert!(code_no_args.contains("pub fn get_config (& self) -> Arc < Config >"));
    assert!(code_no_args.contains("pub fn get_db (& self) -> Arc < Database >"));

    let tokens_async_arg =
        expand_container_attribute(quote!(mode = "async"), &item_struct).unwrap();
    let code_async_arg = tokens_async_arg.to_string();
    assert!(code_async_arg.contains("pub async fn init ()"));
}

#[test]
fn test_expand_derive_container_emits_impls_and_rejects_struct_rewriting_fields() {
    let valid_derive: syn::DeriveInput = parse_quote! {
        pub struct DerivedContainer {
            #[singleton]
            config: Arc<Config>,
        }
    };

    let tokens = expand_derive_container(&valid_derive).unwrap();
    let code = tokens.to_string();
    assert!(!code.contains("pub struct DerivedContainer"));
    assert!(code.contains("impl DerivedContainer"));
    assert!(code.contains("pub fn get_config (& self) -> Arc < Config >"));

    let transient_derive: syn::DeriveInput = parse_quote! {
        pub struct BadDerivedContainer {
            #[transient]
            service: MyService,
        }
    };
    let err = expand_derive_container(&transient_derive).unwrap_err();
    assert!(matches!(err, DomainError::InvalidConfiguration(_)));
}

#[test]
fn test_to_compile_error_formats_domain_errors_as_compiler_diagnostics() {
    let cycle_struct: syn::ItemStruct = parse_quote! {
        pub struct CyclicContainer {
            #[singleton(deps = [b])]
            a: Arc<ServiceA>,

            #[singleton(deps = [a])]
            b: Arc<ServiceB>,
        }
    };

    let err = expand_container_attribute(quote!(), &cycle_struct).unwrap_err();
    let compile_err_tokens = to_compile_error(err).to_string();
    assert!(compile_err_tokens.contains("compile_error !"));
    assert!(compile_err_tokens.contains("dependency cycle detected"));

    let missing_struct: syn::ItemStruct = parse_quote! {
        pub struct MissingDepContainer {
            #[singleton]
            config: Arc<Config>,

            #[singleton(deps = [confog])]
            db: Arc<Database>,
        }
    };

    let missing_err = expand_container_attribute(quote!(), &missing_struct).unwrap_err();
    let missing_tokens = to_compile_error(missing_err).to_string();
    assert!(missing_tokens.contains("compile_error !"));
    assert!(missing_tokens.contains(
        "unknown dependency 'confog' on field 'db'. You must choose between the dependencies the container can wire: [config]"
    ));
}
