//! Tests: what an attribute that does not read still reports.

use syn::parse_quote;

use super::super::*;

#[test]
fn a_bad_deps_entry_is_kept_when_another_setting_does_not_read() {
    let attr: syn::Attribute =
        parse_quote!(#[singleton(deps = [config + 1], bogus, deps = [a::b = config])]);
    let unreadable = DependencyAttrMeta::from_attribute(&attr).unwrap_err();

    assert!(matches!(
        unreadable.problem,
        AttributeProblem::Unreadable(_)
    ));
    assert_eq!(
        unreadable.bad_deps,
        [
            AttributeProblem::NotAField("config + 1".to_string()),
            AttributeProblem::NotALabel("a::b = config".to_string()),
        ]
    );
}
