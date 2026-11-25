use syn::parse_quote;

use super::*;

#[test]
fn a_cfg_and_a_cfg_attr_that_can_write_one_are_conditional() {
    let conditional: [syn::Attribute; 3] = [
        parse_quote!(#[cfg(test)]),
        parse_quote!(#[cfg_attr(all(), cfg(any()))]),
        parse_quote!(#[cfg_attr(unix, cfg_attr(test, cfg(feature = "mocks")))]),
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
