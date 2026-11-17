use super::*;
use crate::domain::{DependencyName, Execution, Lifecycle};
use crate::test_support::{container, dependency};

fn transient(name: &str, execution: Execution) -> DependencyDeclaration<()> {
    let mut dependency = dependency(name, &[]);
    dependency.lifecycle = Lifecycle::Transient;
    dependency.execution = execution;
    dependency
}

#[test]
fn singletons_break_no_rule() {
    let dependencies = vec![dependency("config", &[]), dependency("db", &[])];

    assert!(rule_violations(&container(dependencies)).is_empty());
}

#[test]
fn a_sync_transient_breaks_no_rule() {
    let dependencies = vec![transient("request", Execution::Sync)];

    assert!(rule_violations(&container(dependencies)).is_empty());
}

#[test]
fn an_async_transient_is_refused() {
    let errors = rule_violations(&container(vec![transient("request", Execution::Async)]));

    assert_eq!(
        errors,
        [DomainError::AsyncTransient {
            dependency: DependencyName::new("request").unwrap()
        }]
    );
}
