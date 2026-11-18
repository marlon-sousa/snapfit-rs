use quote::format_ident;
use syn::parse_quote;

use super::*;

fn read(settings: &[syn::Expr]) -> Result<(BuilderMethods, BuilderSyntax), DomainError> {
    methods(
        &DependencyName::new("exporter").unwrap(),
        &format_ident!("builder"),
        settings,
    )
}

fn names(methods: &BuilderMethods) -> (&str, &str) {
    (methods.start.as_str(), methods.finish.as_str())
}

#[test]
fn a_builder_without_settings_starts_with_builder_and_finishes_with_build() {
    let (methods, syntax) = read(&[]).unwrap();

    assert_eq!(names(&methods), ("builder", "build"));
    assert_eq!(syntax.start, format_ident!("builder"));
    assert_eq!(syntax.finish, format_ident!("build"));
}

#[test]
fn start_and_finish_each_replace_their_default_alone_or_together() {
    let cases: [(Vec<syn::Expr>, (&str, &str)); 3] = [
        (vec![parse_quote!(start = make)], ("make", "build")),
        (vec![parse_quote!(finish = done)], ("builder", "done")),
        (
            vec![parse_quote!(finish = done), parse_quote!(start = make)],
            ("make", "done"),
        ),
    ];
    for (settings, expected) in cases {
        let (methods, syntax) = read(&settings).unwrap();
        assert_eq!(names(&methods), expected);
        assert_eq!(
            (syntax.start.to_string(), syntax.finish.to_string()),
            (expected.0.to_string(), expected.1.to_string())
        );
    }
}

#[test]
fn every_setting_that_cannot_be_read_is_refused_in_one_error() {
    let settings: [syn::Expr; 4] = [
        parse_quote!(start = make),
        parse_quote!(begin = make),
        parse_quote!(finish = "done"),
        parse_quote!(start = again),
    ];

    assert_eq!(
        read(&settings).unwrap_err().to_string(),
        "builder(begin = make, finish = \"done\", start = again) on field 'exporter' cannot be read; builder takes start and finish, each with a function's name, such as builder(start = new_builder, finish = done)"
    );
}
