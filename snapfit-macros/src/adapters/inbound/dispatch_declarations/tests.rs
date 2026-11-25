use syn::parse_quote;

use super::*;
use crate::domain::{DomainError, StaticDispatchProblem};

fn read(attrs: Vec<syn::Attribute>) -> Result<(Vec<Declaration>, Vec<Extension>), DomainErrors> {
    read_dispatch_declarations(&attrs)
}

fn names(variants: &[Variant<syn::Type>]) -> Vec<&str> {
    variants
        .iter()
        .map(|variant| variant.name.as_str())
        .collect()
}

fn problems(attrs: Vec<syn::Attribute>) -> Vec<String> {
    read(attrs)
        .unwrap_err()
        .into_iter()
        .map(|error| error.to_string())
        .collect()
}

#[test]
fn a_declaration_reads_its_trait_its_name_and_its_extra_implementations() {
    let (declarations, extensions) = read(vec![
        parse_quote!(#[derive(Debug)]),
        parse_quote!(#[static_dispatch_for(shop::Store<Order> as Orders, extra = [Sqlite, Cache<Order>],)]),
        parse_quote!(#[extend_static_dispatch(Orders, [Mock])]),
        parse_quote!(#[static_dispatch_for(Database as Db)]),
    ])
    .unwrap();

    let [orders, db] = declarations.as_slice() else {
        panic!("two declarations");
    };
    assert_eq!(orders.name.as_str(), "Orders");
    assert_eq!(orders.position, AttributePosition(0));
    let trait_path = &orders.syntax.trait_path;
    assert_eq!(compact(quote::quote!(#trait_path)), "shop::Store<Order>");
    assert_eq!(names(&orders.extra), ["Sqlite", "Cache<Order>"]);
    assert_eq!(db.position, AttributePosition(2));
    assert!(db.extra.is_empty());

    let [extension] = extensions.as_slice() else {
        panic!("one extension");
    };
    assert_eq!(extension.name.as_str(), "Orders");
    assert_eq!(extension.position, AttributePosition(1));
    assert_eq!(names(&extension.variants), ["Mock"]);
}

#[test]
fn an_extension_reads_a_list_a_macro_passes_as_one_value_and_an_empty_one() {
    let list = proc_macro2::Group::new(proc_macro2::Delimiter::None, quote::quote!([Mock]));
    let (_, extensions) = read(vec![
        parse_quote!(#[extend_static_dispatch(Db, #list)]),
        parse_quote!(#[extend_static_dispatch(Db, [])]),
    ])
    .unwrap();

    let [passed, empty] = extensions.as_slice() else {
        panic!("two extensions");
    };
    assert_eq!(names(&passed.variants), ["Mock"]);
    assert!(empty.variants.is_empty());
}

#[test]
fn each_attribute_that_cannot_be_read_is_refused_with_what_it_lacks() {
    let cases: [(syn::Attribute, &str); 12] = [
        (
            parse_quote!(#[static_dispatch_for]),
            "static_dispatch_for cannot be read: it needs its settings in parentheses",
        ),
        (
            parse_quote!(#[static_dispatch_for = "Db"]),
            "static_dispatch_for cannot be read: it takes its settings in parentheses, not after '='",
        ),
        (
            parse_quote!(#[static_dispatch_for()]),
            "static_dispatch_for cannot be read: unexpected end of input",
        ),
        (
            parse_quote!(#[static_dispatch_for(Database)]),
            "static_dispatch_for needs the name the fields write",
        ),
        (
            parse_quote!(#[static_dispatch_for(Database as Db Extra)]),
            "static_dispatch_for cannot be read: expected the word as and the name after the trait",
        ),
        (
            parse_quote!(#[static_dispatch_for(Database as shop::Db)]),
            "static_dispatch_for names the dispatched type through the path shop::Db",
        ),
        (
            parse_quote!(#[static_dispatch_for(Database as Db, extras = [A])]),
            "not extras; write extra = [Sqlite]",
        ),
        (
            parse_quote!(#[static_dispatch_for(Database as Db, Sqlite)]),
            "not Sqlite; write extra = [Sqlite]",
        ),
        (
            parse_quote!(#[static_dispatch_for(Database as Db, extra = (A))]),
            "takes its implementations in square brackets, not (A)",
        ),
        (
            parse_quote!(#[static_dispatch_for(Database as Db, extra = [_])]),
            "_ in static_dispatch_for is not a type the container can name",
        ),
        (
            parse_quote!(#[extend_static_dispatch(Db)]),
            "extend_static_dispatch(Db) needs the implementations it adds",
        ),
        (
            parse_quote!(#[extend_static_dispatch(Db, [A], [B])]),
            "extend_static_dispatch cannot be read: it takes one list after the name, and found [B] after it",
        ),
    ];

    for (attr, expected) in cases {
        let found = problems(vec![attr.clone()]);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0].contains(expected),
            "{} said {found:?}",
            quote::quote!(#attr)
        );
    }
}

#[test]
fn every_problem_of_every_attribute_is_reported_at_its_own_position() {
    let errors: Vec<_> = read(vec![
        parse_quote!(#[static_dispatch_for(Database, extras = [A], extra = [B], extra = [C])]),
        parse_quote!(#[extend_static_dispatch(shop::Db, [A])]),
    ])
    .unwrap_err()
    .into_iter()
    .collect();

    let positions: Vec<_> = errors.iter().map(DomainError::attribute_position).collect();
    assert_eq!(
        positions,
        [0, 0, 0, 1].map(|index| Some(AttributePosition(index)))
    );
    assert!(matches!(
        errors.as_slice(),
        [
            DomainError::StaticDispatch(StaticDispatchProblem::WithoutName { .. }),
            DomainError::StaticDispatch(StaticDispatchProblem::UnknownSetting { .. }),
            DomainError::StaticDispatch(StaticDispatchProblem::RepeatedExtra { .. }),
            DomainError::StaticDispatch(StaticDispatchProblem::NameAsPath { .. }),
        ]
    ));
}
