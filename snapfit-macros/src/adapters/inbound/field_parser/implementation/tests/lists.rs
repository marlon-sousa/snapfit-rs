use proc_macro2::{Delimiter, Group, TokenStream};
use quote::quote;
use syn::parse_quote;

use super::super::super::parse_field;
use super::messages_of;
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
    messages_of(&field(settings))
}

#[test]
fn a_list_in_square_brackets_is_read_as_each_type_it_holds() {
    let entry = through_a_macro(quote!(Cache<u8>));
    let whole = Group::new(Delimiter::None, quote!([Postgres, Sqlite]));
    let cases = [
        (
            "one",
            quote!(implementations = [Postgres]),
            vec!["Postgres"],
        ),
        (
            "several, with a trailing comma",
            quote!(implementations = [Postgres, Cache<u8, u16>,]),
            vec!["Postgres", "Cache<u8, u16>"],
        ),
        (
            "an entry a macro passes",
            quote!(implementations = [Postgres, #entry]),
            vec!["Postgres", "Cache<u8>"],
        ),
        (
            "the whole list a macro passes",
            quote!(implementations = #whole),
            vec!["Postgres", "Sqlite"],
        ),
        (
            "repeated, kept for the policy to count once",
            quote!(implementations = [Postgres, Postgres]),
            vec!["Postgres", "Postgres"],
        ),
    ];

    for (case, settings, expected) in cases {
        assert_eq!(listed(settings), expected, "{case}");
    }
}

#[test]
fn an_empty_list_and_each_entry_that_is_no_type_are_refused_together() {
    let empty = "field 'db': implementations lists no type in its square brackets; list the types the field's factory may return, such as implementations = [Postgres, Sqlite], or remove implementations";
    assert_eq!(messages(quote!(implementations = [])), [empty]);

    assert_eq!(
        messages(quote!(implementations = [Postgres, impl Database, "Sqlite", , _])),
        [
            "field 'db': 'impl Database' is not a type the container can build; write the type in full, such as implementation = StripeGateway",
            "field 'db': implementation takes a type, not a string; write it unquoted: implementation = Sqlite",
            "field 'db': implementation needs a type; write the type the container builds for the field, such as implementation = StripeGateway",
            "field 'db': '_' is not a type the container can build; write the type in full, such as implementation = StripeGateway",
        ]
    );
}

#[test]
fn each_word_refuses_the_other_s_value_and_both_words_or_a_repeated_one_are_refused() {
    let list_in_one = "field 'db': implementation takes one type, not a list; to list the types a factory may return, write implementations, with an s, such as implementations = [Postgres, Sqlite]";
    let one_in_list = "field 'db': implementations takes its types in square brackets, such as implementations = [Postgres, Sqlite]; to name the one type the container builds, write implementation, without an s, such as implementation = Postgres";
    let cases = [
        (
            "a list given to implementation",
            quote!(implementation = [Postgres]),
            vec![list_in_one],
        ),
        (
            "one type given to implementations",
            quote!(implementations = Postgres),
            vec![one_in_list],
        ),
        (
            "square brackets followed by more tokens",
            quote!(implementations = [Postgres] + Send),
            vec![one_in_list],
        ),
        (
            "implementations bare",
            quote!(implementations),
            vec![one_in_list],
        ),
        (
            "both words",
            quote!(implementation = MySql, implementations = [Postgres]),
            vec![
                "field 'db': implementation and implementations are both given; keep implementation for the one type the container builds, or implementations, beside build = factory, for the types its factory may return",
            ],
        ),
        (
            "implementations twice",
            quote!(implementations = [MySql], implementations = [Postgres]),
            vec![
                "field 'db': implementations is given more than once; list every type in one implementations setting",
            ],
        ),
    ];

    for (case, settings, expected) in cases {
        assert_eq!(messages(settings), expected, "{case}");
    }
}
