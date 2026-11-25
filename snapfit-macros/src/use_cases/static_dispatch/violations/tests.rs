use super::*;
use crate::domain::WrittenType;
use crate::domain::{AttributePosition, DependencyName, DomainError, StaticDispatchProblem};
use crate::test_support::dispatched_types::{
    declared, dispatched_field, dispatching_container, extended, type_name,
};
use crate::use_cases::static_dispatch::dispatched_types;

#[test]
fn a_dispatched_type_without_implementations_is_not_planned_and_is_refused() {
    let container = dispatching_container(Vec::new(), vec![declared("Db", 3, &[])], Vec::new());

    assert!(dispatched_types(&container).is_empty());
    assert_eq!(
        dispatch_violations(&container),
        [DomainError::StaticDispatch(
            StaticDispatchProblem::WithoutImplementations {
                position: AttributePosition(3),
                name: type_name("Db"),
            }
        )]
    );
}

#[test]
fn every_problem_of_the_declarations_is_reported_together() {
    let container = dispatching_container(
        vec![dispatched_field(
            "main",
            WrittenType::Path {
                last: type_name("Db"),
                written: "shop::Db".to_owned(),
            },
            Some("MySql"),
        )],
        vec![
            declared("Db", 0, &["MySql"]),
            declared("Db", 1, &["Sqlite"]),
            declared("Cache", 2, &[]),
        ],
        vec![extended("Queue", 3, &["Kafka"])],
    );

    assert_eq!(
        dispatch_violations(&container),
        [
            StaticDispatchProblem::RepeatedName {
                position: AttributePosition(1),
                name: type_name("Db")
            },
            StaticDispatchProblem::Undeclared {
                position: AttributePosition(3),
                name: type_name("Queue"),
                declared: vec![type_name("Db"), type_name("Cache")],
            },
            StaticDispatchProblem::WithoutImplementations {
                position: AttributePosition(2),
                name: type_name("Cache")
            },
            StaticDispatchProblem::ThroughPath {
                dependency: DependencyName::new("main").unwrap(),
                name: type_name("Db"),
                written: "shop::Db".to_owned(),
            },
        ]
        .map(DomainError::StaticDispatch)
    );
}
