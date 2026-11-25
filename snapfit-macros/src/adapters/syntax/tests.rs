use syn::parse_quote;

use super::*;

#[test]
fn a_cfg_and_a_cfg_attr_that_can_write_one_are_conditional() {
    let conditional: [syn::Attribute; 5] = [
        parse_quote!(#[cfg(test)]),
        parse_quote!(#[cfg_attr(all(), cfg(any()))]),
        parse_quote!(#[cfg_attr(unix, cfg_attr(test, cfg(feature = "mocks")))]),
        parse_quote!(#[cfg_attr(true, cfg(false))]),
        parse_quote!(#[cfg_attr(all(), cfg_attr(true, cfg(false)))]),
    ];

    assert!(conditional.iter().all(is_conditional));
}

#[test]
fn a_cfg_attr_that_writes_only_other_attributes_is_not_conditional() {
    let unconditional: [syn::Attribute; 2] = [
        parse_quote!(#[cfg_attr(docsrs, doc = "Reads the cfg of the build.")]),
        parse_quote!(#[cfg_attr(test, allow(dead_code))]),
    ];

    assert!(!unconditional.iter().any(is_conditional));
}

#[test]
fn a_cfg_attr_keeps_only_the_cfg_it_writes() {
    let attr: syn::Attribute = parse_quote!(#[cfg_attr(unix, cfg(test), deprecated = "old", cfg_attr(test, cfg(feature = "mocks"), must_use))]);

    let kept = conditional_part(&attr.meta);

    assert_eq!(
        kept,
        Some(parse_quote!(cfg_attr(
            unix,
            cfg(test),
            cfg_attr(test, cfg(feature = "mocks"))
        )))
    );
}

#[test]
fn a_cfg_that_a_macro_passes_in_a_group_of_its_own_is_read_through_it() {
    let removed = proc_macro2::Group::new(proc_macro2::Delimiter::None, quote::quote!(cfg(any())));
    let attr: syn::Attribute = parse_quote!(#[cfg_attr(all(), #removed)]);

    assert!(is_conditional(&attr));
}

#[test]
fn a_cfg_attr_whose_condition_is_a_boolean_keeps_only_the_cfg_it_writes() {
    let attr: syn::Attribute = parse_quote!(#[cfg_attr(true, cfg(false), deprecated = "old")]);

    let kept = conditional_part(&attr.meta);

    assert_eq!(kept, Some(parse_quote!(cfg_attr(true, cfg(false)))));
}

#[test]
fn a_cfg_attr_that_cannot_be_read_is_kept_whole() {
    let attr: syn::Attribute = parse_quote!(#[cfg_attr(unix cfg(test))]);

    let kept = conditional_part(&attr.meta);

    assert_eq!(kept, Some(attr.meta));
}
