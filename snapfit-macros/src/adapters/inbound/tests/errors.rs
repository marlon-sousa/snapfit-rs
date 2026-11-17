use quote::quote;
use syn::parse_quote;

use super::super::*;

fn messages(errors: DomainErrors) -> Vec<String> {
    errors.into_iter().map(|error| error.to_string()).collect()
}

#[test]
fn every_bad_field_is_reported_at_once() {
    let input: syn::ItemStruct = parse_quote! {
        struct Repo<T> {
            #[singleton(constructor = "connect")]
            db: Arc<Database>,
            config: Arc<Config>,
            #[cfg(test)]
            #[singleton]
            metrics: Metrics,
        }
    };

    let errors = messages(parse_attribute(quote!(), &input).unwrap_err());

    assert_eq!(
        errors,
        [
            "constructor \"connect\" on field 'db' must be written without quotes: constructor = connect",
            "field 'config' must be annotated with #[singleton], #[transient], or #[dispatch]",
            "field 'metrics' cannot use #[cfg] or #[cfg_attr]; conditional dependencies are not supported",
        ]
    );
}

#[test]
fn shapes_that_cannot_be_a_container_are_refused() {
    let invalid_mode: syn::ItemStruct = parse_quote! { struct Bad {} };
    let tuple_struct: syn::ItemStruct = parse_quote! { struct Bad(Arc<Config>); };
    let enum_derive: syn::DeriveInput = parse_quote! { enum Bad { A } };

    assert_eq!(
        messages(parse_attribute(quote!(mode = "threaded"), &invalid_mode).unwrap_err())[0],
        "invalid container mode 'threaded': expected 'sync' or 'async'"
    );
    assert_eq!(
        messages(parse_attribute(quote!(), &tuple_struct).unwrap_err()),
        ["tuple structs are not supported by #[container]; use named fields"]
    );
    assert_eq!(
        messages(parse_derive(&enum_derive).unwrap_err()),
        ["#[derive(Container)] can only be applied to a struct"]
    );
}

#[test]
fn a_second_container_attribute_is_refused() {
    let input: syn::ItemStruct = parse_quote! {
        #[container(mode = "sync")]
        struct App {}
    };

    assert_eq!(
        messages(parse_attribute(quote!(mode = "async"), &input).unwrap_err()),
        ["multiple #[container] attributes are not allowed"]
    );
}
