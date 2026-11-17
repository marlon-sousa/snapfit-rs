use quote::quote;
use syn::parse_quote;

use super::*;

fn item() -> syn::ItemStruct {
    parse_quote! {
        #[derive(Debug)]
        pub struct App {
            /// Settings.
            #[singleton(deps = [missing])]
            pub config: Config,
            plain: Plain,
        }
    }
}

#[test]
fn every_error_becomes_one_compile_error_in_order() {
    let item = item();
    let origins = Origins::new(&item.ident, &item.fields);
    let first = DomainError::TupleStruct;
    let second = DomainError::MultipleContainerAttributes;
    let errors = DomainErrors::check(vec![first, second]).unwrap_err();

    let text = origins.compile_errors(&errors).to_string();

    assert_eq!(text.matches("compile_error").count(), 2);
    assert!(text.find("tuple structs") < text.find("multiple #[container]"));
}

#[test]
fn the_fallback_struct_is_the_item_without_snapfit_attributes() {
    let fallback = fallback_struct(&item()).to_string();

    let expected = quote! {
        #[derive(Debug)]
        pub struct App {
            #[doc = r" Settings."]
            pub config: Config,
            plain: Plain,
        }
    };
    assert_eq!(fallback, expected.to_string());
}
