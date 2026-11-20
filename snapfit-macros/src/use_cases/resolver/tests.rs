use super::*;
use crate::domain::{ConstructionPlan, Execution, Fallibility, Lifecycle, TypeShape};
use crate::test_support::{container, dependency_with, name, names};

mod errors;
mod properties;

fn dependency(text: &str, prerequisites: &[&str]) -> DependencyDeclaration<&'static str> {
    dependency_with(text, prerequisites, "syntax")
}

fn step_names<'a>(steps: &'a [Rc<Step<&'static str>>]) -> Vec<&'a str> {
    steps.iter().map(|step| step.name.as_str()).collect()
}

fn plan(
    container: ContainerDeclaration<(), &'static str>,
    execution: Execution,
) -> ConstructionPlan<(), &'static str> {
    resolve(container)
        .unwrap()
        .into_plan(execution, Fallibility::Infallible)
}

fn errors_of(dependencies: Vec<DependencyDeclaration<&'static str>>) -> Vec<DomainError> {
    resolve(container(dependencies))
        .unwrap_err()
        .into_iter()
        .collect()
}

#[test]
fn every_dependency_is_built_after_its_prerequisites_and_otherwise_in_field_order() {
    let plan = plan(
        container(vec![
            dependency("service", &["db"]),
            dependency("db", &["config"]),
            dependency("config", &[]),
            dependency("clock", &[]),
        ]),
        Execution::Sync,
    );

    assert_eq!(
        step_names(&plan.construction_order),
        ["config", "db", "service", "clock"]
    );
    assert_eq!(
        step_names(&plan.declaration_order),
        ["service", "db", "config", "clock"]
    );
}

#[test]
fn a_step_holds_the_steps_of_its_prerequisites_in_the_order_they_are_passed() {
    let plan = plan(
        container(vec![
            dependency("config", &[]),
            dependency("db", &[]),
            dependency("service", &["db", "config", "db"]),
        ]),
        Execution::Async,
    );

    let service = plan.declaration_order.last().unwrap();
    assert_eq!(step_names(&service.prerequisites), ["db", "config", "db"]);
    assert!(Rc::ptr_eq(
        &service.prerequisites[1],
        &plan.construction_order[0]
    ));
    assert_eq!(plan.execution, Execution::Async);
}

#[test]
fn a_step_keeps_the_declaration_s_syntax_lifecycle_and_execution() {
    let mut request = dependency("request", &[]);
    request.lifecycle = Lifecycle::Transient;
    request.execution = Execution::Async;
    request.shape = TypeShape::Arc;

    let plan = plan(container(vec![request]), Execution::Async);

    let step = &plan.construction_order[0];
    assert_eq!(step.syntax, "syntax");
    assert_eq!(step.lifecycle, Lifecycle::Transient);
    assert_eq!(step.execution, Execution::Async);
}

#[test]
fn a_shared_prerequisite_is_one_step() {
    let plan = plan(
        container(vec![
            dependency("config", &[]),
            dependency("read_db", &["config"]),
            dependency("write_db", &["config"]),
        ]),
        Execution::Sync,
    );

    assert_eq!(plan.construction_order.len(), 3);
    assert!(Rc::ptr_eq(
        &plan.construction_order[1].prerequisites[0],
        &plan.construction_order[2].prerequisites[0]
    ));
}
