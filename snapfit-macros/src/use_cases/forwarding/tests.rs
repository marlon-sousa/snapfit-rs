use super::*;
use crate::domain::{Execution, MethodName, Safety, SelfBound, SelfUse};
use crate::test_support::{first, forwardable_method, method};

mod other_items;

fn database() -> TraitName {
    TraitName::new("Database").unwrap()
}

fn refused(item: TraitItem, problem: ItemProblem) -> DomainError {
    refusal(&database(), &item, ItemPosition::new(0), problem)
}

fn find() -> TraitItem {
    TraitItem::Method(MethodName::new("find").unwrap())
}

#[test]
fn a_method_taking_a_reference_is_forwarded_as_it_returns() {
    assert_eq!(
        forwarding(&database(), first(forwardable_method("find"))),
        Ok(ForwardedItem::Method {
            syntax: "find",
            rebuilding: Rebuilding::None,
            execution: Execution::Sync,
            safety: Safety::Safe,
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
        forwarding(&database(), first(declaration)),
        Ok(ForwardedItem::Method {
            syntax: "renamed",
            rebuilding: Rebuilding::IntoVariant,
            execution: Execution::Sync,
            safety: Safety::Safe,
        })
    );
}

#[test]
fn an_async_unsafe_method_keeps_how_it_is_called() {
    let declaration = TraitItemDeclaration::<&str, &str>::Method {
        name: MethodName::new("find").unwrap(),
        syntax: "find",
        shape: MethodShape::new(
            Receiver::Reference,
            SelfUse::Forwardable,
            SelfBound::Forwardable,
            ReturnShape::Plain,
            Execution::Async,
            Safety::Unsafe,
        ),
    };

    assert_eq!(
        forwarding(&database(), first(declaration)),
        Ok(ForwardedItem::Method {
            syntax: "find",
            rebuilding: Rebuilding::None,
            execution: Execution::Async,
            safety: Safety::Unsafe,
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
            forwarding(
                &database(),
                first(method("find", receiver, self_use, returns))
            ),
            Err(refused(find(), problem).into()),
            "{problem:?}"
        );
    }
}

#[test]
fn a_method_bounding_self_by_more_than_sized_is_refused() {
    let declaration = TraitItemDeclaration::<&str, &str>::Method {
        name: MethodName::new("find").unwrap(),
        syntax: "find",
        shape: MethodShape::new(
            Receiver::Reference,
            SelfUse::Forwardable,
            SelfBound::Other,
            ReturnShape::Plain,
            Execution::Sync,
            Safety::Safe,
        ),
    };

    assert_eq!(
        forwarding(&database(), first(declaration)),
        Err(refused(find(), ItemProblem::SelfBound).into())
    );
}

#[test]
fn a_method_with_several_problems_reports_every_one() {
    let declaration = method(
        "find",
        Receiver::None,
        SelfUse::Elsewhere,
        ReturnShape::ImplTrait,
    );

    let errors: Vec<_> = forwarding(&database(), first(declaration))
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
fn a_refusal_names_the_position_of_its_item() {
    let placed = PlacedItem::new(
        ItemPosition::new(3),
        method(
            "find",
            Receiver::None,
            SelfUse::Forwardable,
            ReturnShape::Plain,
        ),
    );

    assert_eq!(
        forwarding(&database(), placed),
        Err(refusal(
            &database(),
            &find(),
            ItemPosition::new(3),
            ItemProblem::NoReceiver
        )
        .into())
    );
}
