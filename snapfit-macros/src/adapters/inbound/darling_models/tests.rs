use quote::quote;
use syn::parse_quote;

use super::*;

#[test]
fn test_container_attr_meta_parses_all_settings() {
    let attr: syn::Attribute = parse_quote!(#[container(
        mode = "async",
        parent = AppContainer,
        introspection = true
    )]);
    let meta = ContainerAttrMeta::from_attribute(&attr).unwrap();

    assert_eq!(meta.mode.as_deref(), Some("async"));
    assert_eq!(
        meta.parent,
        Some(ParentTypeMeta("AppContainer".to_string()))
    );
    assert!(meta.introspection);
}

#[test]
fn test_container_attr_meta_supports_string_parent_and_empty_tokens() {
    let empty = ContainerAttrMeta::from_tokens(quote!()).unwrap();
    assert_eq!(empty, ContainerAttrMeta::default());

    let meta = ContainerAttrMeta::from_tokens(quote!(parent = "ParentScope")).unwrap();
    assert_eq!(meta.parent, Some(ParentTypeMeta("ParentScope".to_string())));
}

#[test]
fn test_dependency_attr_meta_parses_async_keyword_and_bracket_deps() {
    let attr: syn::Attribute =
        parse_quote!(#[singleton(deps = [config, db], async, constructor = connect)]);
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();

    assert_eq!(meta.deps.0, vec!["config", "db"]);
    assert!(meta.is_async);
    assert_eq!(meta.constructor, Some(FunctionName::Bare("connect".into())));
    assert_eq!(meta.builder, None);
}

#[test]
fn test_dependency_attr_meta_parses_parent_call_and_builder() {
    let attr: syn::Attribute =
        parse_quote!(#[singleton(deps = [parent(db), session], builder = build_logger)]);
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();

    assert_eq!(meta.deps.0, vec!["parent(db)", "session"]);
    assert_eq!(
        meta.builder,
        Some(FunctionName::Bare("build_logger".into()))
    );

    let bad_call: syn::Attribute = parse_quote!(#[singleton(deps = [other(db)])]);
    assert!(DependencyAttrMeta::from_attribute(&bad_call).is_err());
}
