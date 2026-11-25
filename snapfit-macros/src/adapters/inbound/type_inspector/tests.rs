use quote::quote;
use syn::parse_quote;

use super::*;
use crate::test_support::through_a_macro;

#[test]
fn a_type_is_written_compactly_with_its_generics_and_trait_objects() {
    let arc_ty: syn::Type = parse_quote!(std::sync::Arc<Config<Prod>>);
    assert_eq!(
        compact_type_tokens(quote!(#arc_ty)),
        "std::sync::Arc<Config<Prod>>"
    );

    let dyn_ty: syn::Type = parse_quote!(Arc<dyn Database + Send + Sync>);
    assert_eq!(
        compact_type_tokens(quote!(#dyn_ty)),
        "Arc<dyn Database + Send + Sync>"
    );
}

#[test]
fn a_dispatch_attribute_lists_its_implementations_and_refuses_an_empty_list() {
    let attr: syn::Attribute = parse_quote!(#[dispatch(PostgresDb, MockDb)]);
    let impls = parse_dispatch_attribute(&attr).unwrap();
    assert_eq!(impls, vec!["PostgresDb", "MockDb"]);

    let empty_attr: syn::Attribute = parse_quote!(#[dispatch()]);
    assert!(parse_dispatch_attribute(&empty_attr).is_err());

    let path_attr: syn::Attribute = parse_quote!(#[dispatch]);
    assert!(parse_dispatch_attribute(&path_attr).is_err());
}

#[test]
fn the_dispatched_trait_ignores_auto_traits() {
    let ty: syn::Type = parse_quote!(Arc<dyn Send + Sync + Database>);

    assert_eq!(dispatched_trait(&ty).unwrap(), "Database");
}

#[test]
fn a_dispatch_field_without_a_trait_object_is_refused() {
    let ty: syn::Type = parse_quote!(Arc<Database>);

    assert!(dispatched_trait(&ty).is_err());
}

#[test]
fn a_type_holding_dyn_is_a_trait_object_and_an_arc_of_one_is_told_apart() {
    let arc_of_trait_through_a_macro = through_a_macro(quote!(Arc<dyn Store>));
    let arc_through_a_macro = through_a_macro(quote!(Arc<Store>));
    let through_a_macro = through_a_macro(quote!(dyn Store));
    let shapes: [(syn::Type, TypeShape); 10] = [
        (parse_quote!(Arc<dyn Store>), TypeShape::ArcTraitObject),
        (
            parse_quote!(std::sync::Arc<dyn Store + Send + Sync>),
            TypeShape::ArcTraitObject,
        ),
        (
            parse_quote!(Arc<#through_a_macro>),
            TypeShape::ArcTraitObject,
        ),
        (
            parse_quote!(Arc<Archive<dyn Store>>),
            TypeShape::TraitObject,
        ),
        (parse_quote!(Box<dyn Store + Send>), TypeShape::TraitObject),
        (
            parse_quote!(#arc_of_trait_through_a_macro),
            TypeShape::ArcTraitObject,
        ),
        (parse_quote!(Arc<Store>), TypeShape::Arc),
        (parse_quote!(#arc_through_a_macro), TypeShape::Arc),
        (parse_quote!(std::rc::Rc<Store>), TypeShape::Value),
        (parse_quote!(Store), TypeShape::Value),
    ];
    for (ty, shape) in shapes {
        assert_eq!(type_shape(&ty), shape, "{}", quote!(#ty));
    }
}

#[test]
fn only_std_arc_paths_have_an_inner_type() {
    let through_a_macro = through_a_macro(quote!(std::sync::Arc<Config>));
    let arcs: [syn::Type; 5] = [
        parse_quote!(Arc<Config>),
        parse_quote!(#through_a_macro),
        parse_quote!(std::sync::Arc<Config>),
        parse_quote!(::std::sync::Arc<Config>),
        parse_quote!(alloc::sync::Arc<Config>),
    ];
    for ty in &arcs {
        let inner = arc_inner_type(ty).unwrap();
        assert_eq!(compact_type_tokens(quote!(#inner)), "Config");
    }

    let not_arcs: [syn::Type; 5] = [
        parse_quote!(Vec<Config>),
        parse_quote!(std::rc::Rc<Config>),
        parse_quote!(::Arc<Config>),
        parse_quote!(my::Arc<Config>),
        parse_quote!(Arc<Config, Global>),
    ];
    for ty in &not_arcs {
        assert!(arc_inner_type(ty).is_none(), "{}", quote!(#ty));
    }
}
