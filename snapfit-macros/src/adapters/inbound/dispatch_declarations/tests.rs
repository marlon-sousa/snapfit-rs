use syn::parse_quote;

use super::*;
use crate::domain::{DomainError, StaticDispatchProblem, Variant};

mod refusals;

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
fn a_declaration_reads_its_trait_and_its_name() {
    let (declarations, extensions) = read(vec![
        parse_quote!(#[derive(Debug)]),
        parse_quote!(#[static_dispatch_for(shop::Store<Order> as Orders,)]),
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
    assert_eq!(
        compact_type_tokens(quote::quote!(#trait_path)),
        "shop::Store<Order>"
    );
    assert_eq!(db.position, AttributePosition(2));

    let [extension] = extensions.as_slice() else {
        panic!("one extension");
    };
    assert_eq!(extension.name.as_str(), "Orders");
    assert_eq!(extension.position, AttributePosition(1));
    assert_eq!(names(&extension.variants), ["Mock"]);
}

#[test]
fn a_comma_between_angle_brackets_belongs_to_its_type() {
    let (declarations, extensions) = read(vec![
        parse_quote!(#[static_dispatch_for(Store<u8, u16> as Pair)]),
        parse_quote!(#[extend_static_dispatch(Pair, [Boxed<dyn Fn(u8) -> u16, u8>])]),
    ])
    .unwrap();

    let [pair] = declarations.as_slice() else {
        panic!("one declaration");
    };
    let trait_path = &pair.syntax.trait_path;
    assert_eq!(
        compact_type_tokens(quote::quote!(#trait_path)),
        "Store<u8, u16>"
    );
    assert_eq!(
        names(&extensions[0].variants),
        ["Boxed<dyn Fn(u8) -> u16, u8>"]
    );
}

#[test]
fn only_what_reads_is_kept_for_a_refused_container_a_declaration_s_name_without_its_settings() {
    let (declarations, extensions) = read_readable_dispatch_declarations(&[
        parse_quote!(#[static_dispatch_for(Database as Db, extra = [MySql])]),
        parse_quote!(#[static_dispatch_for(Clock as Clocks)]),
        parse_quote!(#[extend_static_dispatch(Clocks)]),
        parse_quote!(#[extend_static_dispatch(Clocks, [Fixed])]),
    ]);

    let names: Vec<_> = declarations
        .iter()
        .map(|declaration| declaration.name.as_str())
        .collect();
    assert_eq!(names, ["Db", "Clocks"]);
    let [extension] = extensions.as_slice() else {
        panic!("one extension");
    };
    assert_eq!(extension.position, AttributePosition(3));
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
