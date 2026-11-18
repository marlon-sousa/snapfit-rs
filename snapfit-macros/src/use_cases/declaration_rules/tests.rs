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

#[test]
fn default_is_refused_with_deps_and_accepted_without() {
    let built_by_default = |deps: &[&str]| {
        let mut rates = dependency("rates", deps);
        rates.construction = Some(Construction::Constructor(Constructor::Default));
        rates
    };

    assert!(rule_violations(&container(vec![built_by_default(&[])])).is_empty());
    assert_eq!(
        rule_violations(&container(vec![built_by_default(&["config"])])),
        [DomainError::DefaultWithDependencies {
            dependency: DependencyName::new("rates").unwrap()
        }]
    );
}
