use proc_macro2::{Delimiter, Group, TokenStream};
use quote::quote;
use syn::parse_quote;

use super::super::super::parse_field;
use crate::domain::Implementation;
use crate::test_support::through_a_macro;

fn field(settings: TokenStream) -> syn::Field {
    parse_quote!(#[singleton(build = factory, #settings)] db: Arc<Db>)
}

fn listed(settings: TokenStream) -> Vec<String> {
    let dependency = parse_field(&field(settings)).unwrap();
    assert_eq!(dependency.syntax.implementation, None);
    let Implementation::Listed(variants) = dependency.implementation else {
        panic!("a list");
    };
    variants
        .iter()
        .map(|variant| variant.name.as_str().to_owned())
        .collect()
}

fn messages(settings: TokenStream) -> Vec<String> {
    super::messages_of(&field(settings))
}

#[test]
fn a_list_in_square_brackets_is_read_as_each_type_it_holds() {
    let entry = through_a_macro(quote!(Cache<u8>));
    let whole = Group::new(Delimiter::None, quote!([Postgres, Sqlite]));
    let cases = [
        ("one", quote!(implementation = [Postgres]), vec!["Postgres"]),
        (
            "several, with a trailing comma",
            quote!(implementation = [Postgres, Cache<u8, u16>,]),
            vec!["Postgres", "Cache<u8, u16>"],
        ),
        (
            "an entry a macro passes",
            quote!(implementation = [Postgres, #entry]),
            vec!["Postgres", "Cache<u8>"],
        ),
        (
            "the whole list a macro passes",
            quote!(implementation = #whole),
            vec!["Postgres", "Sqlite"],
        ),
        (
            "repeated, kept for the policy to count once",
            quote!(implementation = [Postgres, Postgres]),
            vec!["Postgres", "Postgres"],
        ),
    ];

    for (case, settings, expected) in cases {
        assert_eq!(listed(settings), expected, "{case}");
    }
}

#[test]
fn an_empty_list_and_each_entry_that_is_no_type_are_refused_together() {
    let empty = "field 'db': implementation lists no type in its square brackets; write the type the container builds, such as implementation = StripeGateway, or, beside build = factory on a field of a dispatched type, the types its factory may return, such as implementation = [Postgres, Sqlite]";
    assert_eq!(messages(quote!(implementation = [])), [empty]);

    assert_eq!(
        messages(quote!(implementation = [Postgres, impl Database, "Sqlite", , _])),
        [
            "field 'db': 'impl Database' is not a type the container can build; write the type in full, such as implementation = StripeGateway",
            "field 'db': implementation takes a type, not a string; write it unquoted: implementation = Sqlite",
            "field 'db': implementation needs a type; write the type the container builds for the field, such as implementation = StripeGateway",
            "field 'db': '_' is not a type the container can build; write the type in full, such as implementation = StripeGateway",
        ]
    );
}

#[test]
fn square_brackets_followed_by_more_tokens_are_no_list() {
    assert_eq!(
        messages(quote!(implementation = [Postgres] + Send)),
        [
            "field 'db': '[Postgres] + Send' is not a type the container can build; write the type in full, such as implementation = StripeGateway"
        ]
    );
}
