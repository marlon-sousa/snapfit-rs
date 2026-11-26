//! Tests: attribute values that a `macro_rules!` passes as `$e:expr`.

use quote::quote;
use syn::parse_quote;

use super::super::*;
use super::sibling;
use crate::test_support::through_a_macro;

#[test]
fn a_build_passed_through_a_macro_is_read_as_if_written_by_hand() {
    let how = through_a_macro(quote!(default));
    let entry = through_a_macro(quote!(config));
    let meta = DependencyAttrMeta::from_tokens(quote!(deps = [#entry], build = #how)).unwrap();

    assert_eq!(meta.deps.0, vec![sibling("config")]);
    assert_eq!(
        meta.build,
        Some(BuildMeta::Word {
            word: BuildWord::Default,
            written: quote::format_ident!("default"),
        })
    );
}

#[test]
fn every_part_of_a_value_passed_through_a_macro_is_read_as_if_written_by_hand() {
    let parent = through_a_macro(quote!(AppContainer));
    let container = ContainerAttrMeta::from_tokens(quote!(parent = #parent)).unwrap();
    let function = through_a_macro(quote!(connect));
    let label = through_a_macro(quote!(disk));
    let field = through_a_macro(quote!(folder_files));
    let dependency = DependencyAttrMeta::from_tokens(quote!(
        deps = [#label = #field],
        build = constructor(#function)
    ))
    .unwrap();

    assert_eq!(
        container.parent,
        Some(ParentTypeMeta::Named("AppContainer".to_string()))
    );
    assert_eq!(
        dependency.deps.0,
        vec![Ok(PrerequisiteMeta::Labelled {
            label: quote::format_ident!("disk"),
            name: "folder_files".to_string(),
        })]
    );
    assert_eq!(
        dependency.build,
        Some(BuildMeta::Constructor(FunctionNameMeta::Bare(
            quote::format_ident!("connect")
        )))
    );
}

#[test]
fn builder_settings_passed_through_a_macro_are_read_as_if_written_by_hand() {
    let setting = through_a_macro(quote!(start = draft));
    let meta = DependencyAttrMeta::from_tokens(quote!(build = builder(#setting))).unwrap();

    assert_eq!(
        meta.build,
        Some(BuildMeta::Builder {
            word: quote::format_ident!("builder"),
            settings: vec![parse_quote!(start = draft)],
        })
    );
}

#[test]
fn a_parent_dependency_passed_through_a_macro_is_read_as_if_written_by_hand() {
    let parent = through_a_macro(quote!(db));
    let meta = DependencyAttrMeta::from_tokens(quote!(deps = [parent(#parent)])).unwrap();

    assert_eq!(meta.deps.0, vec![Ok(PrerequisiteMeta::Parent("db".into()))]);
}
