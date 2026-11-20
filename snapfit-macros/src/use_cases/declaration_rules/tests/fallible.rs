//! Tests: the rules a field marked `fallible = true` breaks when what would
//! build it cannot fail.

use super::super::*;
use crate::domain::{DependencyName, Execution, Fallibility, FunctionName, InfallibleBuild};
use crate::test_support::{container, dependency};

#[test]
fn default_is_refused_on_a_fallible_field_because_it_cannot_fail() {
    let mut rates = dependency("rates", &[]);
    rates.construction = Some(Construction::Constructor(Constructor::Default));
    rates.fallibility = Fallibility::Fallible;

    assert_eq!(
        rule_violations(&container(vec![rates])),
        [DomainError::BuildCannotFail {
            dependency: DependencyName::new("rates").unwrap(),
            build: InfallibleBuild::Default,
        }]
    );
}

fn fallible(name: &str, prerequisites: &[&str]) -> DependencyDeclaration<()> {
    let mut dependency = dependency(name, prerequisites);
    dependency.fallibility = Fallibility::Fallible;
    dependency
}

#[test]
fn a_fallible_leaf_is_refused_when_leaf_build_would_build_it_because_it_cannot_fail() {
    let mut app = container(vec![fallible("config", &[]), fallible("db", &["config"])]);
    app.leaf_constructor = Constructor::Default;

    assert_eq!(
        rule_violations(&app),
        [DomainError::BuildCannotFail {
            dependency: DependencyName::new("config").unwrap(),
            build: InfallibleBuild::LeafBuild,
        }]
    );
}

#[test]
fn a_fallible_leaf_with_its_own_build_or_under_new_breaks_no_rule() {
    let mut own_build = fallible("config", &[]);
    own_build.construction = Some(Construction::Constructor(Constructor::New));
    let mut app = container(vec![own_build]);
    app.leaf_constructor = Constructor::Default;
    assert!(rule_violations(&app).is_empty());

    assert!(rule_violations(&container(vec![fallible("config", &[])])).is_empty());
}

#[test]
fn a_fallible_leaf_under_leaf_build_written_constructor_new_breaks_no_rule() {
    let mut app = container(vec![fallible("config", &[])]);
    app.leaf_constructor = Constructor::Named(FunctionName::new("new").unwrap());

    assert!(rule_violations(&app).is_empty());
}

#[test]
fn an_async_fallible_leaf_breaks_no_rule_because_leaf_build_never_builds_it() {
    let mut warehouse = fallible("warehouse", &[]);
    warehouse.execution = Execution::Async;
    let mut app = container(vec![warehouse]);
    app.leaf_constructor = Constructor::Default;

    assert!(rule_violations(&app).is_empty());
}
