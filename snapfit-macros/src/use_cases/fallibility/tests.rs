use super::*;
use crate::test_support::{container, dependency};

#[test]
fn a_container_can_fail_to_build_only_when_a_dependency_can() {
    let mut db = dependency("db", &["config"]);
    db.fallibility = Fallibility::Fallible;
    let infallible = container(vec![
        dependency("config", &[]),
        dependency("db", &["config"]),
    ]);
    let fallible = container(vec![dependency("config", &[]), db]);

    assert_eq!(container_fallibility(&infallible), Fallibility::Infallible);
    assert_eq!(container_fallibility(&fallible), Fallibility::Fallible);
}
