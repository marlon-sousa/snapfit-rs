use quote::quote;

use super::super::parse_field;
use crate::test_support::through_a_macro;

/// The messages of the field `cache` with `settings` in its `#[singleton(...)]`.
fn messages(settings: proc_macro2::TokenStream) -> Vec<String> {
    let field: syn::Field = syn::parse_quote!(#[singleton(#settings)] cache: Cache);
    parse_field(&field)
        .unwrap_err()
        .into_iter()
        .map(|error| error.to_string())
        .collect()
}

const UNKNOWN_BOGUS: &str =
    "field 'cache': Unknown field: `bogus`. Available values: `async`, `build`, `deps`, `fallible`";

fn unknown_build(word: &str) -> String {
    format!(
        "build = {word} on field 'cache' is not a way to build a field; write build = default, build = factory, build = builder, or build = constructor({word}) to call a function of that name"
    )
}

#[test]
fn a_bad_setting_is_reported_beside_a_setting_that_does_not_read() {
    let word = through_a_macro(quote!(frobnicate));
    let fallible = "field 'cache': fallible takes true or false, not maybe; write fallible = true, or fallible = false";
    let cases = [
        (
            "build",
            quote!(build = frobnicate, bogus),
            unknown_build("frobnicate"),
        ),
        (
            "fallible",
            quote!(bogus, fallible = maybe),
            fallible.to_string(),
        ),
        (
            "build through a macro",
            quote!(bogus, build = #word),
            unknown_build("frobnicate"),
        ),
    ];
    for (case, settings, expected) in cases {
        assert_eq!(
            messages(settings),
            [UNKNOWN_BOGUS.to_string(), expected],
            "{case}"
        );
    }
}

#[test]
fn every_bad_setting_is_reported_beside_several_that_do_not_read() {
    assert_eq!(
        messages(quote!(
            build = "default",
            fallible,
            deps = [a + 1],
            bogus,
            other
        )),
        [
            "field 'cache': Multiple errors: (Unknown field: `bogus`. Available values: `async`, `build`, `deps`, `fallible`, Unknown field: `other`. Available values: `async`, `build`, `deps`, `fallible`)",
            "build \"default\" on field 'cache' must be written without quotes: build = default",
            "field 'cache': fallible needs a value; write fallible = true to have the container return the field's error, or leave fallible out",
            "field 'cache': the deps entry 'a + 1' is not the name of a field; write another field's name, such as config",
        ]
    );
}

#[test]
fn each_bad_build_of_a_repeated_build_is_reported_beside_the_repetition() {
    assert_eq!(
        messages(quote!(build = frobnicate, build = sideways)),
        [
            "field 'cache': Duplicate field `build`".to_string(),
            unknown_build("frobnicate"),
            unknown_build("sideways"),
        ]
    );
}

#[test]
fn good_settings_add_nothing_to_a_setting_that_does_not_read() {
    assert_eq!(
        messages(quote!(
            build = default,
            fallible = true,
            deps = [config],
            bogus
        )),
        [UNKNOWN_BOGUS]
    );
}
