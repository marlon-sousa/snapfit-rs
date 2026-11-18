use quote::format_ident;
use syn::parse_quote;

use super::super::darling_models::{ContainerAttrMeta, DependencyAttrMeta};
use super::*;

type FieldBuild = Result<(Option<Construction>, Option<syn::Ident>), DomainError>;

/// `build = ...` of a `#[singleton(...)]` on a field named `db`.
fn field_build(attr: syn::Attribute) -> FieldBuild {
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();
    field_construction(&DependencyName::new("db").unwrap(), meta.build)
}

fn message(attr: syn::Attribute) -> String {
    field_build(attr).unwrap_err().to_string()
}

fn leaf(build: syn::Expr) -> Result<(Constructor, Option<syn::Ident>), DomainError> {
    let meta = ContainerAttrMeta::from_tokens(quote::quote!(leaf_build = #build)).unwrap();
    leaf_constructor(meta.leaf_build)
}

#[test]
fn each_build_word_reads_as_its_way_of_building() {
    let cases: [(syn::Attribute, Option<Construction>); 5] = [
        (parse_quote!(#[singleton]), None),
        (
            parse_quote!(#[singleton(build = default)]),
            Some(Construction::Constructor(Constructor::Default)),
        ),
        (
            parse_quote!(#[transient(deps = [config], build = factory)]),
            Some(Construction::Factory),
        ),
        (
            parse_quote!(#[singleton(build = builder)]),
            Some(Construction::Builder),
        ),
        (
            parse_quote!(#[singleton(build = builder(start = make))]),
            Some(Construction::Builder),
        ),
    ];
    for (attr, construction) in cases {
        assert_eq!(field_build(attr).unwrap(), (construction, None));
    }
}

#[test]
fn a_named_constructor_keeps_the_user_s_own_token() {
    let (construction, written) =
        field_build(parse_quote!(#[singleton(build = constructor(connect))])).unwrap();

    assert_eq!(
        construction,
        Some(Construction::Constructor(Constructor::Named(
            FunctionName::new("connect").unwrap()
        )))
    );
    assert_eq!(written, Some(format_ident!("connect")));
}

#[test]
fn a_build_value_that_is_no_way_of_building_is_refused_with_what_to_write() {
    let cases: [(syn::Attribute, &str); 5] = [
        (
            parse_quote!(#[singleton(build = factroy)]),
            "build = factroy on field 'db' is not a way to build a field; write build = default, build = factory, or build = constructor(factroy) to call a function of that name",
        ),
        (
            parse_quote!(#[singleton(build = "factory")]),
            "build \"factory\" on field 'db' must be written without quotes: build = factory",
        ),
        (
            parse_quote!(#[singleton(build = "default")]),
            "build \"default\" on field 'db' must be written without quotes: build = default",
        ),
        (
            parse_quote!(#[singleton(build = Db::connect)]),
            "build = Db :: connect on field 'db' is not a way to build a field; write build = default, build = factory, or build = constructor(connect)",
        ),
        (
            parse_quote!(#[singleton(build = constructor)]),
            "build = constructor on field 'db' needs the function's name, such as build = constructor(connect)",
        ),
    ];
    for (attr, expected) in cases {
        assert_eq!(message(attr), expected);
    }
}

#[test]
fn a_function_name_must_be_written_bare() {
    assert_eq!(
        message(parse_quote!(#[singleton(build = constructor("connect"))])),
        "constructor \"connect\" on field 'db' must be written without quotes: build = constructor(connect)"
    );

    let cases: [(syn::Attribute, &str); 5] = [
        (
            parse_quote!(#[singleton(build = constructor("connect()"))]),
            "connect()",
        ),
        (
            parse_quote!(#[singleton(build = constructor(connect()))]),
            "connect ()",
        ),
        (
            parse_quote!(#[singleton(build = constructor(Db::connect))]),
            "Db :: connect",
        ),
        (
            parse_quote!(#[singleton(build = constructor(open, connect))]),
            "open , connect",
        ),
        (
            parse_quote!(#[singleton(build = constructor("   "))]),
            "   ",
        ),
    ];
    for (attr, shown) in cases {
        assert_eq!(
            message(attr),
            format!(
                "constructor '{shown}' on field 'db' must be a function name, such as build = constructor(connect)"
            )
        );
    }
}

#[test]
fn leaf_build_takes_default_or_a_constructor_and_nothing_else() {
    assert_eq!(leaf_constructor(None), Ok((Constructor::New, None)));
    assert_eq!(
        leaf(parse_quote!(default)),
        Ok((Constructor::Default, None))
    );
    assert_eq!(
        leaf(parse_quote!(constructor(open))),
        Ok((
            Constructor::Named(FunctionName::new("open").unwrap()),
            Some(format_ident!("open"))
        ))
    );

    let refused = [
        (parse_quote!(factory), "factory"),
        (parse_quote!(builder), "builder"),
        (parse_quote!("default"), "\"default\""),
        (
            parse_quote!(constructor(Db::open)),
            "constructor(Db :: open)",
        ),
    ];
    for (build, written) in refused {
        assert_eq!(
            leaf(build),
            Err(DomainError::InvalidLeafBuild {
                written: written.to_string()
            })
        );
    }
}
