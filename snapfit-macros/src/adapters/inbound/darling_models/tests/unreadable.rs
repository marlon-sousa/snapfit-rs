//! Tests: what an attribute that does not read still reports.

use quote::{format_ident, quote};
use syn::parse_quote;

use super::super::*;

#[test]
fn each_setting_that_reads_alone_is_kept_when_another_does_not_read() {
    let attr: syn::Attribute = parse_quote!(#[singleton(
        deps = [config + 1],
        bogus,
        build = frobnicate,
        fallible = maybe,
        deps = [a::b = config]
    )]);
    let unreadable = DependencyAttrMeta::from_attribute(&attr).unwrap_err();

    assert!(matches!(unreadable.problem, ArgumentProblem::Unreadable(_)));
    let kept = unreadable.settings;
    assert_eq!(kept.len(), 4, "{kept:?}");
    assert_eq!(
        kept[0].deps.0,
        [Err(ArgumentProblem::NotAField("config + 1".to_string()))]
    );
    assert_eq!(
        kept[1].build,
        Some(BuildMeta::UnknownWord(format_ident!("frobnicate")))
    );
    assert_eq!(
        kept[2].fallible,
        Some(FallibleMeta::NotABoolean("maybe".to_string()))
    );
    assert_eq!(
        kept[3].deps.0,
        [Err(ArgumentProblem::NotALabel("a::b = config".to_string()))]
    );
}

#[test]
fn a_setting_that_does_not_read_alone_is_left_to_darlings_problem() {
    let attr: syn::Attribute = parse_quote!(#[singleton(build(default), fallible(true), bogus)]);
    let unreadable = DependencyAttrMeta::from_attribute(&attr).unwrap_err();

    assert_eq!(
        unreadable.settings,
        [DependencyAttrMeta {
            fallible: Some(FallibleMeta::InParentheses),
            ..DependencyAttrMeta::default()
        }]
    );
}

#[test]
fn each_container_setting_that_reads_alone_is_kept_when_another_does_not_read() {
    let unreadable =
        ContainerAttrMeta::from_tokens(quote!(leaf_build = frobnicate, bogus, mode = "turbo"))
            .unwrap_err();

    assert!(matches!(unreadable.problem, ArgumentProblem::Unreadable(_)));
    assert_eq!(
        unreadable.settings,
        [
            ContainerAttrMeta {
                leaf_build: Some(BuildMeta::UnknownWord(format_ident!("frobnicate"))),
                ..ContainerAttrMeta::default()
            },
            ContainerAttrMeta {
                mode: Some(ModeMeta::Written("turbo".to_string())),
                ..ContainerAttrMeta::default()
            },
        ]
    );
}

#[test]
fn a_setting_written_again_is_refused_after_the_rest_reads() {
    let unreadable =
        ContainerAttrMeta::from_tokens(quote!(mode = "sync", mode = "async", crate = di, crate))
            .unwrap_err();

    assert_eq!(
        unreadable.problem,
        ArgumentProblem::RepeatedSetting("mode".to_string())
    );
    assert_eq!(
        unreadable.repeated,
        [ArgumentProblem::RepeatedSetting("crate".to_string())]
    );
    assert_eq!(unreadable.settings.len(), 3, "{:?}", unreadable.settings);
    assert_eq!(
        unreadable.settings[1].mode,
        Some(ModeMeta::Written("async".to_string()))
    );
    assert_eq!(unreadable.settings[2].runtime, Some(RuntimeMeta::Bare));
}

#[test]
fn a_setting_written_again_is_refused_beside_darlings_problem() {
    let unreadable =
        ContainerAttrMeta::from_tokens(quote!(bogus, mode = "sync", mode = 3)).unwrap_err();

    assert!(matches!(unreadable.problem, ArgumentProblem::Unreadable(_)));
    assert_eq!(unreadable.repeated.len(), 2, "{:?}", unreadable.repeated);
    assert_eq!(
        unreadable.repeated[0],
        ArgumentProblem::RepeatedSetting("mode".to_string())
    );
    assert!(matches!(
        unreadable.repeated[1],
        ArgumentProblem::Unreadable(_)
    ));
    assert_eq!(
        unreadable.settings,
        [ContainerAttrMeta {
            mode: Some(ModeMeta::Written("sync".to_string())),
            ..ContainerAttrMeta::default()
        }]
    );
}
