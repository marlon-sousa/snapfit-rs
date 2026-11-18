use super::*;
use crate::domain::{DependencyName, Execution, Lifecycle, SetterName};
use crate::test_support::{builder, container, dependency, labelled};

mod properties;

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

#[test]
fn default_on_an_async_dependency_is_refused() {
    let mut rates = dependency("rates", &[]);
    rates.construction = Some(Construction::Constructor(Constructor::Default));
    rates.execution = Execution::Async;

    assert_eq!(
        rule_violations(&container(vec![rates])),
        [DomainError::AsyncDefault {
            dependency: DependencyName::new("rates").unwrap()
        }]
    );
}

#[test]
fn a_label_on_a_field_not_built_by_a_builder_is_refused_for_each_label() {
    let mut writer = dependency("writer", &["config"]);
    writer.prerequisites.extend([
        labelled("folder_files", "disk"),
        labelled("memory_files", "memory"),
    ]);

    let errors: Vec<_> = rule_violations(&container(vec![writer]))
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(
        errors,
        [
            "the dependency 'disk = folder_files' of field 'writer' has a label, which names a builder setter; write folder_files alone, or build the field with build = builder",
            "the dependency 'memory = memory_files' of field 'writer' has a label, which names a builder setter; write memory_files alone, or build the field with build = builder",
        ]
    );
}

#[test]
fn a_builder_takes_labels_and_refuses_a_setter_passed_twice_once() {
    let mut writer = dependency("writer", &["disk"]);
    writer.construction = Some(builder());
    writer.prerequisites.extend([
        labelled("folder_files", "disk"),
        labelled("memory_files", "memory"),
        labelled("spare_files", "disk"),
    ]);

    assert_eq!(
        rule_violations(&container(vec![writer])),
        [DomainError::DuplicateSetter {
            dependency: DependencyName::new("writer").unwrap(),
            setter: SetterName::new("disk").unwrap(),
        }]
    );
}

#[test]
fn a_builder_with_a_setter_for_each_dependency_breaks_no_rule() {
    let mut writer = dependency("writer", &["config"]);
    writer.construction = Some(builder());
    writer.prerequisites.push(labelled("folder_files", "disk"));

    assert!(rule_violations(&container(vec![writer])).is_empty());
}

#[test]
fn a_field_passed_again_to_the_same_setter_is_refused_once() {
    let mut writer = dependency("writer", &["config", "db", "config", "config"]);
    writer.construction = Some(builder());
    writer.prerequisites.push(labelled("db", "db"));

    let errors: Vec<_> = rule_violations(&container(vec![writer]))
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(
        errors,
        [
            "field 'writer' lists 'config' in deps more than once; list each dependency once",
            "field 'writer' lists 'db' in deps more than once; list each dependency once",
        ]
    );
}

#[test]
fn a_builder_may_pass_one_field_to_two_setters() {
    let mut store = dependency("store", &[]);
    store.construction = Some(builder());
    store
        .prerequisites
        .extend([labelled("pool", "reader"), labelled("pool", "writer")]);

    assert!(rule_violations(&container(vec![store])).is_empty());
}

#[test]
fn a_field_passed_twice_to_one_setter_is_a_repeat_not_a_second_use_of_the_setter() {
    let mut writer = dependency("writer", &[]);
    writer.construction = Some(builder());
    writer.prerequisites.extend([
        labelled("folder_files", "disk"),
        labelled("folder_files", "disk"),
    ]);

    assert_eq!(
        rule_violations(&container(vec![writer])),
        [DomainError::RepeatedPrerequisite {
            dependency: DependencyName::new("writer").unwrap(),
            prerequisite: DependencyName::new("folder_files").unwrap(),
        }]
    );
}
