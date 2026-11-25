use syn::parse_quote;

use super::*;
use crate::domain::SelfUse;

#[test]
fn a_plain_associated_type_is_forwardable_in_every_way() {
    let associated: syn::TraitItemType = parse_quote!(
        type Row: Clone;
    );

    assert_eq!(
        associated_type_shape(&associated),
        AssociatedTypeShape::new(TypeParameters::None, SelfUse::Forwardable, Presence::Always)
    );
}

#[test]
fn parameters_self_in_a_bound_and_a_cfg_are_each_read() {
    let cases: [(syn::TraitItemType, AssociatedTypeShape); 4] = [
        (
            parse_quote!(
                type Rows<'a>;
            ),
            AssociatedTypeShape::new(
                TypeParameters::Declared,
                SelfUse::Forwardable,
                Presence::Always,
            ),
        ),
        (
            parse_quote!(
                type Twin: Into<Self>;
            ),
            AssociatedTypeShape::new(TypeParameters::None, SelfUse::Elsewhere, Presence::Always),
        ),
        (
            parse_quote!(
                #[cfg(test)]
                type Row;
            ),
            AssociatedTypeShape::new(
                TypeParameters::None,
                SelfUse::Forwardable,
                Presence::Conditional,
            ),
        ),
        (
            parse_quote!(
                #[cfg_attr(test, allow(unused))]
                type Row;
            ),
            AssociatedTypeShape::new(
                TypeParameters::None,
                SelfUse::Forwardable,
                Presence::Conditional,
            ),
        ),
    ];
    for (associated, shape) in cases {
        let written = quote::quote!(#associated).to_string();
        assert_eq!(associated_type_shape(&associated), shape, "{written}");
    }
}
