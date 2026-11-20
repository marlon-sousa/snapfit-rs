use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use super::super::parse_field;
use crate::domain::TypeShape;
use crate::test_support::through_a_macro;

/// The field `reader` with `settings` in its `#[singleton(...)]`.
fn field(settings: TokenStream) -> syn::Field {
    parse_quote!(#[singleton(#settings)] reader: Arc<dyn Reader>)
}

fn messages(settings: TokenStream) -> Vec<String> {
    parse_field(&field(settings))
        .unwrap_err()
        .into_iter()
        .map(|error| error.to_string())
        .collect()
}

fn implementation(settings: TokenStream) -> Option<syn::Type> {
    parse_field(&field(settings)).unwrap().syntax.implementation
}

#[test]
fn implementation_is_read_as_the_type_written_beside_the_other_settings() {
    let disk: syn::Type = parse_quote!(DiskReader);
    let cases = [
        ("left out", quote!(), None),
        ("alone", quote!(implementation = DiskReader), Some(&disk)),
        (
            "with a type of several parameters",
            quote!(implementation = Cache<Order, Clock>),
            Some(&parse_quote!(Cache<Order, Clock>)),
        ),
        (
            "between other settings",
            quote!(
                build = constructor(open),
                implementation = DiskReader,
                fallible = true
            ),
            Some(&disk),
        ),
    ];

    for (case, settings, expected) in cases {
        assert_eq!(implementation(settings).as_ref(), expected, "{case}");
    }
}

#[test]
fn implementation_written_through_a_macro_is_read_as_its_type() {
    let through_a_macro = through_a_macro(quote!(Cache<Order>));

    let read = implementation(quote!(implementation = #through_a_macro)).unwrap();

    assert_eq!(
        quote!(#read).to_string(),
        quote!(#through_a_macro).to_string()
    );
}

#[test]
fn a_field_naming_an_implementation_is_marked_named_and_keeps_its_arc_shape() {
    let dependency =
        parse_field(&field(quote!(implementation = DiskReader, fallible = true))).unwrap();

    assert!(dependency.implementation.is_named());
    assert_eq!(dependency.shape, TypeShape::ArcTraitObject);
    assert!(dependency.fallibility.is_fallible());
}

#[test]
fn implementation_without_a_type_written_out_after_the_equals_sign_is_refused_at_the_field() {
    let not_a_type = |written: &str| {
        format!(
            "field 'reader': '{written}' is not a type the container can build; write the type in full, such as implementation = StripeGateway"
        )
    };
    let bare = "field 'reader': implementation needs a type; write the type the container builds for the field, such as implementation = StripeGateway".to_owned();
    let cases = [
        ("bare", quote!(implementation), bare.clone()),
        ("empty parentheses", quote!(implementation()), bare.clone()),
        ("empty string", quote!(implementation = ""), bare),
        (
            "in parentheses",
            quote!(implementation(DiskReader)),
            "field 'reader': 'implementation(DiskReader)' writes its type in parentheses; write it after an equals sign: implementation = DiskReader".to_owned(),
        ),
        (
            "quoted",
            quote!(implementation = "DiskReader"),
            "field 'reader': implementation takes a type, not a string; write it unquoted: implementation = DiskReader".to_owned(),
        ),
        ("impl Trait", quote!(implementation = impl Reader), not_a_type("impl Reader")),
        ("impl Trait inside", quote!(implementation = Cache<impl Reader>), not_a_type("Cache<impl Reader>")),
        ("inferred", quote!(implementation = _), not_a_type("_")),
        ("inferred inside", quote!(implementation = Cache<_>), not_a_type("Cache<_>")),
        (
            "inferred beside another parameter",
            quote!(implementation = HashMap<_, String>),
            not_a_type("HashMap<_, String>"),
        ),
        (
            "impl Trait beside another parameter",
            quote!(implementation = Cache<impl Reader, u8>),
            not_a_type("Cache<impl Reader, u8>"),
        ),
        ("a trait object", quote!(implementation = dyn Reader), not_a_type("dyn Reader")),
        ("an expression", quote!(implementation = 3 + 4), not_a_type("3 + 4")),
    ];

    for (case, settings, expected) in cases {
        assert_eq!(messages(settings), [expected], "{case}");
    }
}

#[test]
fn every_problem_of_implementation_is_reported_with_the_field_s_other_problems() {
    assert_eq!(
        messages(quote!(
            implementation = DiskReader,
            implementation = "DiskReader",
            fallible = "yes"
        )),
        [
            "field 'reader': fallible takes true or false, not \"yes\"; write fallible = true, or fallible = false",
            "field 'reader': implementation takes a type, not a string; write it unquoted: implementation = DiskReader",
            "field 'reader': implementation is given more than once; give the field one type to build",
        ]
    );
}

#[test]
fn a_bad_implementation_is_reported_beside_a_setting_darling_cannot_read() {
    let reported = messages(quote!(implementation = impl Reader, unknown = 3));

    assert_eq!(reported.len(), 2, "{reported:?}");
    assert!(
        reported[0].contains("Unknown field: `unknown`"),
        "{reported:?}"
    );
    assert_eq!(
        reported[1],
        "field 'reader': 'impl Reader' is not a type the container can build; write the type in full, such as implementation = StripeGateway"
    );
}

#[test]
fn a_misspelt_setting_is_offered_implementation() {
    let reported = messages(quote!(implementaton = DiskReader));

    assert!(
        reported[0].contains("Did you mean `implementation`?"),
        "{reported:?}"
    );
}
