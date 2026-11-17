use super::*;
use crate::domain::DependencyName;
use crate::test_support;

/// `config`, `db` and `cache`, of which `async_names` are async.
fn container(mode: Option<Execution>, async_names: &[&str]) -> ContainerDeclaration<(), ()> {
    let mut container = test_support::container(
        ["config", "db", "cache"]
            .into_iter()
            .map(|name| declared(name, async_names))
            .collect(),
    );
    container.mode = mode;
    container
}

fn declared(name: &str, async_names: &[&str]) -> DependencyDeclaration<()> {
    let mut dependency = test_support::dependency(name, &[]);
    if async_names.contains(&name) {
        dependency.execution = Execution::Async;
    }
    dependency
}

#[test]
fn a_container_with_no_mode_and_no_async_dependency_is_synchronous() {
    assert_eq!(
        container_execution(&container(None, &[])),
        Ok(Execution::Sync)
    );
}

#[test]
fn a_container_with_no_mode_is_async_when_a_dependency_is() {
    assert_eq!(
        container_execution(&container(None, &["db"])),
        Ok(Execution::Async)
    );
}

#[test]
fn a_container_declared_async_is_async_without_async_dependencies() {
    assert_eq!(
        container_execution(&container(Some(Execution::Async), &[])),
        Ok(Execution::Async)
    );
}

#[test]
fn a_container_declared_sync_reports_every_async_dependency() {
    let errors =
        container_execution(&container(Some(Execution::Sync), &["db", "cache"])).unwrap_err();

    let subjects: Vec<_> = errors
        .iter()
        .filter_map(DomainError::subject)
        .map(DependencyName::as_str)
        .collect();
    assert_eq!(subjects, ["db", "cache"]);
}
