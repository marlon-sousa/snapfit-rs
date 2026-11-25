use super::*;
use crate::domain::{ConstantName, MacroName, MethodName, SelfUse, TypeName};
use crate::test_support::{forwardable_method, method};

fn database() -> TraitName {
    TraitName::new("Database").unwrap()
}

fn refused(item: TraitItem, problem: ItemProblem) -> DomainError {
    refusal(&database(), &item, problem)
}

fn find() -> TraitItem {
    TraitItem::Method(MethodName::new("find").unwrap())
}

#[test]
fn a_method_taking_a_reference_is_forwarded_as_it_returns() {
    assert_eq!(
        forwarding(&database(), forwardable_method("find")),
        Ok(ForwardedItem::Method {
            syntax: "find",
            rebuilding: Rebuilding::None,
        })
    );
}

#[test]
fn a_method_returning_self_is_rebuilt_into_its_variant() {
    let declaration = method(
        "renamed",
        Receiver::Value,
        SelfUse::Forwardable,
        ReturnShape::ItSelf,
    );

    assert_eq!(
        forwarding(&database(), declaration),
        Ok(ForwardedItem::Method {
            syntax: "renamed",
            rebuilding: Rebuilding::IntoVariant,
        })
    );
}

#[test]
fn each_shape_a_match_cannot_forward_is_refused_with_its_own_reason() {
    let cases = [
        (
            Receiver::None,
            SelfUse::Forwardable,
            ReturnShape::Plain,
            ItemProblem::NoReceiver,
        ),
        (
            Receiver::Other,
            SelfUse::Forwardable,
            ReturnShape::Plain,
            ItemProblem::OtherReceiver,
        ),
        (
            Receiver::Reference,
            SelfUse::Elsewhere,
            ReturnShape::Plain,
            ItemProblem::SelfElsewhere,
        ),
        (
            Receiver::Reference,
            SelfUse::Forwardable,
            ReturnShape::ImplTrait,
            ItemProblem::ImplTraitReturn,
        ),
    ];
    for (receiver, self_use, returns, problem) in cases {
        assert_eq!(
            forwarding(&database(), method("find", receiver, self_use, returns)),
            Err(refused(find(), problem).into()),
            "{problem:?}"
        );
    }
}

#[test]
fn a_method_with_several_problems_reports_every_one() {
    let declaration = method(
        "find",
        Receiver::None,
        SelfUse::Elsewhere,
        ReturnShape::ImplTrait,
    );

    let errors: Vec<_> = forwarding(&database(), declaration)
        .unwrap_err()
        .into_iter()
        .collect();

    assert_eq!(
        errors,
        [
            refused(find(), ItemProblem::NoReceiver),
            refused(find(), ItemProblem::SelfElsewhere),
            refused(find(), ItemProblem::ImplTraitReturn),
        ]
    );
}

#[test]
fn an_associated_type_is_forwarded_unless_it_is_generic() {
    let rows = || TypeName::new("Rows").unwrap();
    let plain = TraitItemDeclaration::<&str, &str>::AssociatedType {
        name: rows(),
        syntax: "Rows",
        parameters: TypeParameters::None,
    };
    let generic = TraitItemDeclaration::<&str, &str>::AssociatedType {
        name: rows(),
        syntax: "Rows",
        parameters: TypeParameters::Declared,
    };

    assert_eq!(
        forwarding(&database(), plain),
        Ok(ForwardedItem::AssociatedType("Rows"))
    );
    assert_eq!(
        forwarding(&database(), generic),
        Err(refused(
            TraitItem::AssociatedType(rows()),
            ItemProblem::GenericAssociatedType
        )
        .into())
    );
}

#[test]
fn an_associated_const_and_a_macro_call_are_refused() {
    let limit = ConstantName::new("LIMIT").unwrap();
    let items = MacroName::new("items").unwrap();

    assert_eq!(
        forwarding(
            &database(),
            TraitItemDeclaration::<&str, &str>::AssociatedConst(limit.clone())
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
            TraitItemDeclaration::<&str, &str>::Macro(items.clone())
        ),
        Err(refused(TraitItem::Macro(items), ItemProblem::Macro).into())
    );
}
