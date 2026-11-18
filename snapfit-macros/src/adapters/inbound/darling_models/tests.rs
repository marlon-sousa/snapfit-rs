use quote::quote;
use syn::parse_quote;

use super::*;

mod macro_input;

fn sibling(name: &str) -> PrerequisiteMeta {
    PrerequisiteMeta::Sibling {
        name: name.to_string(),
        word: Some(quote::format_ident!("{name}")),
    }
}

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

    assert_eq!(meta.deps.0, vec![sibling("config"), sibling("db")]);
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
        vec![PrerequisiteMeta::Parent("db".into()), sibling("session")]
    );
    assert_eq!(
        meta.build,
        Some(BuildMeta::Word {
            word: BuildWord::Factory,
            written: quote::format_ident!("factory"),
        })
    );

    let bad_call: syn::Attribute = parse_quote!(#[singleton(deps = [other(db)])]);
    assert!(DependencyAttrMeta::from_attribute(&bad_call).is_err());
}

#[test]
fn a_build_value_keeps_every_shape_the_user_can_write() {
    let cases: [(syn::Attribute, BuildMeta); 5] = [
        (
            parse_quote!(#[singleton(build = default)]),
            BuildMeta::Word {
                word: BuildWord::Default,
                written: quote::format_ident!("default"),
            },
        ),
        (
            parse_quote!(#[singleton(build = factroy)]),
            BuildMeta::UnknownWord(quote::format_ident!("factroy")),
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

#[test]
fn a_labelled_dependency_reads_its_setter_and_its_field() {
    let attr: syn::Attribute =
        parse_quote!(#[singleton(deps = [disk = folder_files, "config"], build = builder)]);
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();

    assert_eq!(
        meta.deps.0,
        vec![
            PrerequisiteMeta::Labelled {
                label: quote::format_ident!("disk"),
                name: "folder_files".to_string(),
            },
            sibling("config"),
        ]
    );
}

#[test]
fn builder_reads_its_word_and_its_settings_as_written() {
    let attr: syn::Attribute = parse_quote!(#[singleton(build = builder(start = make))]);
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();

    assert_eq!(
        meta.build,
        Some(BuildMeta::Builder {
            word: quote::format_ident!("builder"),
            settings: vec![parse_quote!(start = make)],
        })
    );
}

#[test]
fn darling_reads_a_build_word_only_from_a_string_so_the_bare_word_is_handed_to_it() {
    assert_eq!(
        BuildWord::from_string("factory").ok(),
        Some(BuildWord::Factory)
    );
    assert!(BuildWord::from_expr(&parse_quote!(factory)).is_err());
    assert_eq!(
        BuildMeta::from_expr(&parse_quote!(factory)).ok(),
        Some(BuildMeta::Word {
            word: BuildWord::Factory,
            written: quote::format_ident!("factory"),
        })
    );
}

#[test]
fn a_dependency_attribute_knows_async_by_its_own_name_only() {
    let hidden: syn::Attribute = parse_quote!(#[singleton(is_async)]);
    let removed: syn::Attribute = parse_quote!(#[singleton(constructor = connect)]);

    assert!(DependencyAttrMeta::from_attribute(&hidden).is_err());
    assert!(matches!(
        DependencyAttrMeta::from_attribute(&removed),
        Err(AttributeProblem::Unreadable(message))
            if message.contains("`async`") && !message.contains("is_async")
    ));
}
