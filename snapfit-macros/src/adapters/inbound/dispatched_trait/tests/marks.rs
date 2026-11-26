use quote::quote;
use syn::parse_quote;

use super::*;

#[test]
fn a_trait_that_another_mark_follows_is_read_as_marked_again_through_any_path_to_snapfit() {
    let followed_by = [
        quote!(#[snapfit::static_dispatch]),
        quote!(#[static_dispatch]),
        quote!(#[di::static_dispatch(crate = di)]),
    ];

    for mark in followed_by {
        let item: syn::Item = parse_quote!(
            #[doc = "Where the orders are kept."]
            #mark
            pub trait r#Database {
                fn create() -> Self;
            }
        );

        let attribute = read(quote!(), item);

        let Annotated::MarkedAgain(name) = attribute.annotated else {
            panic!("the trait is marked again");
        };
        assert_eq!(name.to_string(), "Database");
    }
}

#[test]
fn a_trait_with_other_attributes_alone_is_read_as_a_trait() {
    let attribute = read(
        quote!(),
        parse_quote!(
            #[doc = "Where the orders are kept."]
            #[static_dispatch_for(Database as Db)]
            pub trait Database {
                fn find(&self) -> u32;
            }
        ),
    );

    assert!(attribute.annotated.is_trait());
}
