use syn::parse_quote;

use super::*;

#[test]
fn each_attribute_that_cannot_be_read_is_refused_with_what_it_lacks() {
    let cases: [(syn::Attribute, &str); 13] = [
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
            "static_dispatch_for cannot be read: it found Extra after the name, where only a closing comma may follow",
        ),
        (
            parse_quote!(#[static_dispatch_for(Database as shop::Db)]),
            "static_dispatch_for names the dispatched type through the path shop::Db",
        ),
        (
            parse_quote!(#[static_dispatch_for(Database as Db, extra = [A])]),
            "static_dispatch_for takes the trait and its name and nothing else, not extra; remove extra",
        ),
        (
            parse_quote!(#[static_dispatch_for(Database as Db, Sqlite)]),
            "nothing else, not Sqlite; remove Sqlite",
        ),
        (
            parse_quote!(#[extend_static_dispatch(Db, (A))]),
            "extend_static_dispatch takes its implementations in square brackets, not (A)",
        ),
        (
            parse_quote!(#[extend_static_dispatch(Db, [_])]),
            "_ in extend_static_dispatch is not a type the container can name",
        ),
        (
            parse_quote!(#[extend_static_dispatch(shop::Db, [A])]),
            "extend_static_dispatch names the dispatched type through the path shop::Db",
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
        parse_quote!(#[static_dispatch_for(Database, extras = [A], extra = [B])]),
        parse_quote!(#[extend_static_dispatch(shop::Db, [impl Database])]),
    ])
    .unwrap_err()
    .into_iter()
    .collect();

    let positions: Vec<_> = errors.iter().map(DomainError::attribute_position).collect();
    assert_eq!(
        positions,
        [0, 0, 0, 1, 1].map(|index| Some(AttributePosition(index)))
    );
    assert!(matches!(
        errors.as_slice(),
        [
            DomainError::StaticDispatch(StaticDispatchProblem::WithoutName { .. }),
            DomainError::StaticDispatch(StaticDispatchProblem::UnknownSetting { .. }),
            DomainError::StaticDispatch(StaticDispatchProblem::UnknownSetting { .. }),
            DomainError::StaticDispatch(StaticDispatchProblem::NameAsPath { .. }),
            DomainError::StaticDispatch(StaticDispatchProblem::NotAType { .. }),
        ]
    ));
}
