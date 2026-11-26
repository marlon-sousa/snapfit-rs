use super::*;
use crate::domain::{
    DispatchArguments, DispatchSetting, Execution, ForwardedItem, Rebuilding, Receiver,
    ReturnShape, Safety, SelfUse, TraitName,
};
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
                rebuilding: Rebuilding::None,
                execution: Execution::Sync,
                safety: Safety::Safe,
            },
            ForwardedItem::Method {
                syntax: "count",
                rebuilding: Rebuilding::None,
                execution: Execution::Sync,
                safety: Safety::Safe,
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
    attribute.arguments = DispatchArguments::new(vec![DispatchSetting::Other]);

    let errors = messages(plan_dispatch(attribute).unwrap_err());

    assert_eq!(errors.len(), 3);
    assert!(errors[0].contains("takes no arguments"));
    assert!(errors[1].starts_with("method 'create' in trait 'Database' has no receiver"));
    assert!(errors[2].starts_with("method 'orders' in trait 'Database' returns impl Trait"));
}

#[test]
fn a_trait_marked_again_is_refused_beside_its_arguments_alone() {
    let name = TraitName::new("Database".to_string()).unwrap();
    let attribute = DispatchAttribute::<(), &str, &str>::new(
        DispatchArguments::new(vec![DispatchSetting::Other]),
        Annotated::MarkedAgain(name),
    );

    let errors = messages(plan_dispatch(attribute).unwrap_err());

    assert_eq!(errors.len(), 2);
    assert!(
        errors[0]
            .starts_with("trait 'Database' carries #[snapfit::static_dispatch] more than once")
    );
    assert!(errors[1].contains("takes no arguments"));
}

#[test]
fn an_attribute_on_something_other_than_a_trait_is_refused_beside_its_arguments() {
    let attribute = DispatchAttribute::<(), &str, &str>::new(
        DispatchArguments::new(vec![DispatchSetting::Other]),
        Annotated::Other,
    );

    let errors = messages(plan_dispatch(attribute).unwrap_err());

    assert_eq!(errors.len(), 2);
    assert!(errors[0].starts_with("#[snapfit::static_dispatch] goes on a trait"));
    assert!(errors[1].contains("takes no arguments"));
}
