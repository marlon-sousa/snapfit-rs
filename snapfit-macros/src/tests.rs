use quote::quote;
use syn::parse_quote;

use super::*;

const STABLE: FeatureGate = FeatureGate::new(false);
const UNSTABLE: FeatureGate = FeatureGate::new(true);

fn expand(
    args: proc_macro2::TokenStream,
    item: &syn::ItemStruct,
    gate: FeatureGate,
) -> Result<proc_macro2::TokenStream, DomainError> {
    expand_container_attribute(args, item, &syntax_of_struct(item), gate)
}

fn expand_derive(
    input: &syn::DeriveInput,
    gate: FeatureGate,
) -> Result<proc_macro2::TokenStream, DomainError> {
    expand_derive_container(input, &syntax_of_derive_input(input), gate)
}

fn compact(tokens: impl ToString) -> String {
    tokens.to_string().split_whitespace().collect()
}

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

    let code_no_args = compact(expand(quote!(), &item_struct, STABLE).unwrap());
    assert!(code_no_args.contains("pubstructAppContainer"));
    assert!(code_no_args.contains("pubfnnew()->Self"));
    assert!(code_no_args.contains("pubfnget_config(&self)->Arc<Config>"));
    assert!(code_no_args.contains("pubfnget_db(&self)->Arc<Database>"));

    let code_async_arg = compact(expand(quote!(mode = "async"), &item_struct, UNSTABLE).unwrap());
    assert!(code_async_arg.contains("pubasyncfninit()"));

    let stable_async_err = expand(quote!(mode = "async"), &item_struct, STABLE).unwrap_err();
    assert!(matches!(
        stable_async_err,
        DomainError::RequiresUnstable { .. }
    ));
}

#[test]
fn test_expand_derive_container_emits_impls_and_rejects_struct_rewriting_fields() {
    let valid_derive: syn::DeriveInput = parse_quote! {
        pub struct DerivedContainer {
            #[singleton]
            config: Arc<Config>,
        }
    };

    let code = compact(expand_derive(&valid_derive, STABLE).unwrap());
    assert!(!code.contains("pubstructDerivedContainer"));
    assert!(code.contains("implDerivedContainer"));
    assert!(code.contains("pubfnget_config(&self)->Arc<Config>"));

    let transient_derive: syn::DeriveInput = parse_quote! {
        pub struct BadDerivedContainer {
            #[transient]
            service: MyService,
        }
    };
    let err = expand_derive(&transient_derive, UNSTABLE).unwrap_err();
    assert!(matches!(err, DomainError::InvalidConfiguration(_)));
}

#[test]
fn test_generic_containers_are_refused_until_designed() {
    let generic: syn::ItemStruct = parse_quote! {
        pub struct Repo<T: Clone> {
            #[singleton]
            store: T,
        }
    };

    let err = expand(quote!(), &generic, UNSTABLE).unwrap_err();
    assert_eq!(
        err.to_string(),
        "generic parameters on container 'Repo' is planned for a future version of snapfit and is not implemented yet"
    );
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

    let err = expand(quote!(), &cycle_struct, STABLE).unwrap_err();
    let compile_err_tokens = to_compile_error(&syntax_of_struct(&cycle_struct), err).to_string();
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

    let missing_err = expand(quote!(), &missing_struct, STABLE).unwrap_err();
    let missing_tokens =
        to_compile_error(&syntax_of_struct(&missing_struct), missing_err).to_string();
    assert!(missing_tokens.contains("compile_error !"));
    assert!(missing_tokens.contains(
        "unknown dependency 'confog' on field 'db'. You must choose between the dependencies the container can wire: [config]"
    ));
}

#[test]
fn test_fallback_struct_keeps_the_struct_without_snapfit_attributes() {
    let item: syn::ItemStruct = parse_quote! {
        #[derive(Debug)]
        pub struct Broken<T> {
            #[singleton(deps = [missing])]
            pub value: T,
        }
    };

    assert_eq!(
        compact(fallback_struct(&syntax_of_struct(&item))),
        "#[derive(Debug)]pubstructBroken<T>{pubvalue:T,}"
    );
}
