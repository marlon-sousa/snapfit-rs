//! Tests: a setting written with no value is read, for snapfit to refuse in
//! its own words.

use quote::quote;
use syn::parse_quote;

use super::super::*;

#[test]
fn every_container_setting_that_takes_a_value_is_read_bare() {
    let meta = ContainerAttrMeta::from_tokens(quote!(mode, parent, leaf_build, crate)).unwrap();

    assert_eq!(meta.mode, Some(ModeMeta::Bare));
    assert_eq!(meta.parent, Some(ParentTypeMeta::Bare));
    assert_eq!(meta.leaf_build, Some(BuildMeta::Bare));
    assert_eq!(meta.runtime, Some(RuntimeMeta::Bare));
}

#[test]
fn a_bare_build_on_a_field_is_read_bare() {
    let attr: syn::Attribute = parse_quote!(#[singleton(build)]);
    let meta = DependencyAttrMeta::from_attribute(&attr).unwrap();

    assert_eq!(meta.build, Some(BuildMeta::Bare));
}
