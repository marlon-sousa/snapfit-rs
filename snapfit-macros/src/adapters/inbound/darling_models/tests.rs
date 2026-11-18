use quote::quote;
use syn::parse_quote;

use super::*;

#[test]
fn the_container_attribute_reads_every_setting() {
    let meta = ContainerAttrMeta::from_tokens(quote!(
        mode = "async",
        parent = AppContainer,
        introspection = true
    ))
    .unwrap();

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
        parse_quote!(#[singleton(deps = [config, db], async, build = constructor(connect))]);
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
        meta.build,
        Some(BuildMeta::Constructor(FunctionNameMeta::Bare(
            quote::format_ident!("connect")
        )))
    );
}

#[test]
fn a_dependency_attribute_reads_parent_deps_and_a_factory() {
    let attr: syn::Attribute =
        parse_quote!(#[singleton(deps = [parent(db), session], build = factory)]);
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();

    assert_eq!(
        meta.deps.0,
        vec![
            PrerequisiteMeta::Parent("db".into()),
            PrerequisiteMeta::Sibling("session".into())
        ]
    );
    assert_eq!(
        meta.build,
        Some(BuildMeta::Word(quote::format_ident!("factory")))
    );

    let bad_call: syn::Attribute = parse_quote!(#[singleton(deps = [other(db)])]);
    assert!(DependencyAttrMeta::from_attribute(&bad_call).is_err());
}

#[test]
fn a_build_value_keeps_every_shape_the_user_can_write() {
    let cases: [(syn::Attribute, BuildMeta); 4] = [
        (
            parse_quote!(#[singleton(build = default)]),
            BuildMeta::Word(quote::format_ident!("default")),
        ),
        (
            parse_quote!(#[singleton(build = constructor("connect"))]),
            BuildMeta::Constructor(FunctionNameMeta::Quoted("connect".into())),
        ),
        (
            parse_quote!(#[singleton(build = "factory")]),
            BuildMeta::Quoted("factory".into()),
        ),
        (
            parse_quote!(#[singleton(build = Db::connect)]),
            BuildMeta::Invalid("Db :: connect".into()),
        ),
    ];
    for (attr, expected) in cases {
        let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();
        assert_eq!(meta.build, Some(expected));
    }
}
