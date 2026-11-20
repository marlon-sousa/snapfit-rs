use quote::{format_ident, quote};
use syn::parse_quote;

use super::super::parse_field;
use crate::domain::Fallibility;
use crate::test_support::through_a_macro;

/// The field `db` with `settings` in its `#[singleton(...)]`.
fn field(settings: proc_macro2::TokenStream) -> syn::Field {
    parse_quote!(#[singleton(#settings)] db: Database)
}

fn messages(settings: proc_macro2::TokenStream) -> Vec<String> {
    parse_field(&field(settings))
        .unwrap_err()
        .into_iter()
        .map(|error| error.to_string())
        .collect()
}

#[test]
fn fallible_is_read_as_the_boolean_it_is_given_and_false_when_left_out() {
    let yes = through_a_macro(quote!(true));
    let no = through_a_macro(quote!(false));
    let cases = [
        ("left out", quote!(), Fallibility::Infallible),
        ("false", quote!(fallible = false), Fallibility::Infallible),
        (
            "false through a macro",
            quote!(fallible = #no),
            Fallibility::Infallible,
        ),
        ("true", quote!(fallible = true), Fallibility::Fallible),
        (
            "true through a macro",
            quote!(fallible = #yes),
            Fallibility::Fallible,
        ),
    ];

    for (case, settings, expected) in cases {
        let dependency = parse_field(&field(settings)).unwrap();
        assert_eq!(dependency.fallibility, expected, "{case}");
    }
}

#[test]
fn only_a_fallible_field_keeps_its_fallible_word_for_the_span_of_its_wrapping() {
    let fallible = parse_field(&field(quote!(fallible = true))).unwrap();
    let infallible = parse_field(&field(quote!(fallible = false))).unwrap();

    assert_eq!(fallible.syntax.fallible, Some(format_ident!("fallible")));
    assert_eq!(infallible.syntax.fallible, None);
}

#[test]
fn fallible_without_a_boolean_is_refused_at_the_field_asking_for_one() {
    let cases = [
        (
            "bare",
            quote!(fallible),
            "field 'db': fallible needs a value; write fallible = true to have the container return the field's error, or leave fallible out",
        ),
        (
            "quoted",
            quote!(fallible = "yes"),
            "field 'db': fallible takes true or false, not \"yes\"; write fallible = true, or fallible = false",
        ),
        (
            "in parentheses",
            quote!(fallible(true)),
            "field 'db': fallible takes its value after '=', not in parentheses; write fallible = true, or fallible = false",
        ),
    ];

    for (case, settings, expected) in cases {
        assert_eq!(messages(settings), [expected], "{case}");
    }
}

#[test]
fn fallible_given_twice_is_refused_by_darling() {
    let errors = messages(quote!(fallible = true, fallible = false));

    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Duplicate field `fallible`"),
        "{errors:?}"
    );
}

#[test]
fn a_bare_fallible_is_reported_with_the_field_s_other_problems() {
    let errors = messages(quote!(fallible, build = factroy));

    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(errors.iter().any(|error| error.contains("factroy")));
    assert!(
        errors
            .iter()
            .any(|error| error.contains("fallible needs a value"))
    );
}
