use super::*;
use crate::domain::{DispatchArguments, ForwardedItem, Rebuilding, Receiver, ReturnShape, SelfUse};
use crate::test_support::{dispatch_attribute, forwardable_method, method};

fn messages(errors: DomainErrors) -> Vec<String> {
    errors.into_iter().map(|error| error.to_string()).collect()
}

#[test]
fn a_trait_of_forwardable_items_is_planned_in_the_order_it_declares_them() {
    let plan = plan_dispatch(dispatch_attribute(vec![
        forwardable_method("find"),
        forwardable_method("count"),
    ]))
    .unwrap();

    assert_eq!(
        plan.items,
        [
            ForwardedItem::Method {
                syntax: "find",
                rebuilding: Rebuilding::None
            },
            ForwardedItem::Method {
                syntax: "count",
                rebuilding: Rebuilding::None
            },
        ]
    );
}

#[test]
fn every_problem_of_the_attribute_and_of_each_item_is_reported_together() {
    let mut attribute = dispatch_attribute(vec![
        method(
            "create",
            Receiver::None,
            SelfUse::Forwardable,
            ReturnShape::ItSelf,
        ),
        forwardable_method("find"),
        method(
            "orders",
            Receiver::Reference,
            SelfUse::Forwardable,
            ReturnShape::ImplTrait,
        ),
    ]);
    attribute.arguments = DispatchArguments::Other;

    let errors = messages(plan_dispatch(attribute).unwrap_err());

    assert_eq!(errors.len(), 3);
    assert!(errors[0].contains("takes no arguments"));
    assert!(errors[1].starts_with("method 'create' in trait 'Database' has no receiver"));
    assert!(errors[2].starts_with("method 'orders' in trait 'Database' returns impl Trait"));
}

#[test]
fn an_attribute_on_something_other_than_a_trait_is_refused_beside_its_arguments() {
    let attribute =
        DispatchAttribute::<(), &str, &str>::new(DispatchArguments::Other, Annotated::Other);

    let errors = messages(plan_dispatch(attribute).unwrap_err());

    assert_eq!(errors.len(), 2);
    assert!(errors[0].starts_with("#[snapfit::static_dispatch] goes on a trait"));
    assert!(errors[1].contains("takes no arguments"));
}
