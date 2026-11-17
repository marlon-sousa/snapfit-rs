use quote::quote;
use syn::parse_quote;

use super::*;

#[test]
fn the_container_attribute_reads_every_setting() {
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
fn the_container_attribute_reads_a_quoted_parent_and_no_settings() {
    let empty = ContainerAttrMeta::from_tokens(quote!()).unwrap();
    assert_eq!(empty, ContainerAttrMeta::default());

    let meta = ContainerAttrMeta::from_tokens(quote!(parent = "ParentScope")).unwrap();
    assert_eq!(meta.parent, Some(ParentTypeMeta("ParentScope".to_string())));
}

#[test]
fn a_dependency_attribute_reads_the_async_keyword_and_bracketed_deps() {
    let attr: syn::Attribute =
        parse_quote!(#[singleton(deps = [config, db], async, constructor = connect)]);
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();

    assert_eq!(
        meta.deps.0,
        vec![
            PrerequisiteMeta::Sibling("config".into()),
            PrerequisiteMeta::Sibling("db".into())
        ]
    );
    assert!(meta.is_async);
    assert_eq!(
        meta.constructor,
        Some(FunctionNameMeta::Bare("connect".into()))
    );
    assert_eq!(meta.builder, None);
}

#[test]
fn a_dependency_attribute_reads_parent_deps_and_a_builder() {
    let attr: syn::Attribute =
        parse_quote!(#[singleton(deps = [parent(db), session], builder = build_logger)]);
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();

    assert_eq!(
        meta.deps.0,
        vec![
            PrerequisiteMeta::Parent("db".into()),
            PrerequisiteMeta::Sibling("session".into())
        ]
    );
    assert_eq!(
        meta.builder,
        Some(FunctionNameMeta::Bare("build_logger".into()))
    );

    let bad_call: syn::Attribute = parse_quote!(#[singleton(deps = [other(db)])]);
    assert!(DependencyAttrMeta::from_attribute(&bad_call).is_err());
}
