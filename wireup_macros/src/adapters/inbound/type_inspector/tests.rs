use quote::quote;
use syn::parse_quote;

use super::*;

#[test]
fn test_compact_type_tokens_formats_generics_and_trait_objects() {
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
fn test_parse_dispatch_attribute_extracts_impls_and_rejects_empty() {
    let attr: syn::Attribute = parse_quote!(#[dispatch(PostgresDb, MockDb)]);
    let impls = parse_dispatch_attribute(&attr).unwrap();
    assert_eq!(impls, vec!["PostgresDb", "MockDb"]);

    let empty_attr: syn::Attribute = parse_quote!(#[dispatch()]);
    assert!(parse_dispatch_attribute(&empty_attr).is_err());

    let path_attr: syn::Attribute = parse_quote!(#[dispatch]);
    assert!(parse_dispatch_attribute(&path_attr).is_err());
}

#[test]
fn test_resolve_trait_dispatch_ignores_auto_traits_for_enum_and_detects_dyn() {
    let ty: syn::Type = parse_quote!(Arc<dyn Send + Sync + Database>);
    let enum_dispatch = resolve_trait_dispatch(
        &ty,
        Some(vec!["PostgresDb".to_string(), "MockDb".to_string()]),
    )
    .unwrap();
    assert_eq!(
        enum_dispatch,
        TraitDispatch::Enum {
            trait_name: "Database".to_string(),
            impls: vec!["PostgresDb".to_string(), "MockDb".to_string()],
        }
    );

    let dyn_dispatch = resolve_trait_dispatch(&ty, None).unwrap();
    assert_eq!(
        dyn_dispatch,
        TraitDispatch::Dyn {
            trait_type: "dyn Send + Sync + Database".to_string(),
        }
    );
}
