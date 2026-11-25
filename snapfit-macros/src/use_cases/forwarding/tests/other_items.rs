//! Associated types, associated consts and macro calls.

use super::*;
use crate::domain::{ConstantName, MacroName, Presence, SelfBound, TypeName};

fn rows() -> TypeName {
    TypeName::new("Rows").unwrap()
}

fn associated_type(shape: AssociatedTypeShape) -> TraitItemDeclaration<&'static str, &'static str> {
    TraitItemDeclaration::AssociatedType {
        name: rows(),
        syntax: "Rows",
        shape,
    }
}

fn plain_type() -> AssociatedTypeShape {
    AssociatedTypeShape::new(
        TypeParameters::None,
        SelfUse::Forwardable,
        SelfBound::Forwardable,
        Presence::Always,
    )
}

#[test]
fn a_plain_associated_type_is_forwarded() {
    assert_eq!(
        forwarding(&database(), first(associated_type(plain_type()))),
        Ok(ForwardedItem::AssociatedType("Rows"))
    );
}

#[test]
fn each_associated_type_a_bound_cannot_agree_on_is_refused_with_its_own_reason() {
    let cases = [
        (
            AssociatedTypeShape::new(
                TypeParameters::Declared,
                SelfUse::Forwardable,
                SelfBound::Forwardable,
                Presence::Always,
            ),
            ItemProblem::GenericAssociatedType,
        ),
        (
            AssociatedTypeShape::new(
                TypeParameters::None,
                SelfUse::Elsewhere,
                SelfBound::Forwardable,
                Presence::Always,
            ),
            ItemProblem::SelfInBounds,
        ),
        (
            AssociatedTypeShape::new(
                TypeParameters::None,
                SelfUse::Forwardable,
                SelfBound::Other,
                Presence::Always,
            ),
            ItemProblem::SelfBound,
        ),
        (
            AssociatedTypeShape::new(
                TypeParameters::None,
                SelfUse::Forwardable,
                SelfBound::Forwardable,
                Presence::Conditional,
            ),
            ItemProblem::ConditionalAssociatedType,
        ),
    ];
    for (shape, problem) in cases {
        assert_eq!(
            forwarding(&database(), first(associated_type(shape))),
            Err(refused(TraitItem::AssociatedType(rows()), problem).into()),
            "{problem:?}"
        );
    }
}

#[test]
fn an_associated_const_and_a_macro_call_are_refused() {
    let limit = ConstantName::new("LIMIT").unwrap();
    let items = MacroName::new("items").unwrap();

    assert_eq!(
        forwarding(
            &database(),
            first(TraitItemDeclaration::AssociatedConst(limit.clone()))
        ),
        Err(refused(
            TraitItem::AssociatedConst(limit),
            ItemProblem::AssociatedConst
        )
        .into())
    );
    assert_eq!(
        forwarding(
            &database(),
            first(TraitItemDeclaration::Macro(items.clone()))
        ),
        Err(refused(TraitItem::Macro(items), ItemProblem::Macro).into())
    );
}
