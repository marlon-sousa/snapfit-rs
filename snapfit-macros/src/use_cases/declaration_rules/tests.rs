use super::*;
use crate::domain::{DependencyName, EnumDispatch, Execution, Lifecycle, TypeName};
use crate::test_support::{self, dependency};

fn container(
    form: MacroForm,
    dependencies: Vec<DependencyDeclaration<()>>,
) -> ContainerDeclaration<(), ()> {
    let mut container = test_support::container(dependencies);
    container.form = form;
    container
}

fn transient(name: &str, execution: Execution) -> DependencyDeclaration<()> {
    let mut dependency = dependency(name, &[]);
    dependency.lifecycle = Lifecycle::Transient;
    dependency.execution = execution;
    dependency
}

#[test]
fn singletons_break_no_rule_in_either_form() {
    let dependencies = vec![dependency("config", &[]), dependency("db", &[])];

    assert!(rule_violations(&container(MacroForm::Attribute, dependencies.clone())).is_empty());
    assert!(rule_violations(&container(MacroForm::Derive, dependencies)).is_empty());
}

#[test]
fn an_async_transient_is_refused() {
    let errors = rule_violations(&container(
        MacroForm::Attribute,
        vec![transient("request", Execution::Async)],
    ));

    assert_eq!(
        errors,
        [DomainError::AsyncTransient {
            dependency: DependencyName::new("request").unwrap()
        }]
    );
}

#[test]
fn the_derive_reports_every_field_it_cannot_leave_out() {
    let mut dispatched = dependency("store", &[]);
    let store = TypeName::new("Store").unwrap();
    dispatched.dispatch = Some(EnumDispatch::new(store.clone(), vec![store]));

    let errors = rule_violations(&container(
        MacroForm::Derive,
        vec![
            transient("request", Execution::Sync),
            dependency("config", &[]),
            dispatched,
        ],
    ));

    let subjects: Vec<_> = errors
        .iter()
        .filter_map(DomainError::subject)
        .map(DependencyName::as_str)
        .collect();
    assert_eq!(subjects, ["request", "store"]);
}

#[test]
fn an_async_transient_in_a_derive_breaks_both_rules() {
    let errors = rule_violations(&container(
        MacroForm::Derive,
        vec![transient("request", Execution::Async)],
    ));

    assert_eq!(errors.len(), 2);
}
