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
            #[singleton(build = constructor("connect"))]
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
            "constructor \"connect\" on field 'db' must be written without quotes: build = constructor(connect)",
            "field 'config' has no snapfit attribute; add #[singleton] to have the container build it",
            "field 'metrics' cannot use #[cfg] or #[cfg_attr]; conditional dependencies are not supported",
        ]
    );
}

#[test]
fn shapes_that_cannot_be_a_container_are_refused() {
    let invalid_mode: syn::ItemStruct = parse_quote! { struct Bad {} };
    let tuple_struct: syn::ItemStruct = parse_quote! { struct Bad(Arc<Config>); };

    assert_eq!(
        messages(parse_attribute(quote!(mode = "threaded"), &invalid_mode).unwrap_err())[0],
        "invalid container mode 'threaded': write mode = \"sync\", or leave mode out"
    );
    assert_eq!(
        messages(parse_attribute(quote!(), &tuple_struct).unwrap_err()),
        ["tuple structs are not supported by #[container]; use named fields"]
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

#[test]
fn a_container_attribute_after_the_macro_is_a_second_declaration() {
    let input: syn::ItemStruct = parse_quote! {
        #[container(mode = "sync")]
        struct App {}
    };

    assert_eq!(
        messages(parse_attribute(quote!(), &input).unwrap_err()),
        ["multiple #[container] attributes are not allowed"]
    );
}
