use super::*;
use crate::test_support::{cycle, name};

#[test]
fn a_cycle_reads_as_a_sentence_without_arrows() {
    let error = cycle("service_a", &["service_b", "service_c"]);

    assert_eq!(
        error.to_string(),
        "dependency cycle: service_a depends on service_b, which depends on service_c, which depends on service_a"
    );
}

#[test]
fn a_dependency_that_needs_itself_is_named_once() {
    let error = cycle("service_a", &[]);

    assert_eq!(
        error.to_string(),
        "dependency cycle: service_a depends on itself"
    );
}

#[test]
fn an_unknown_prerequisite_names_what_the_container_can_wire() {
    let error = DomainError::UnknownPrerequisites {
        dependency: name("db"),
        missing: vec![name("confog")],
        available: vec![name("config"), name("user_service")],
    };

    assert_eq!(
        error.to_string(),
        "unknown dependency 'confog' on field 'db'. You must choose between the dependencies the container can wire: [config, user_service]"
    );
    assert_eq!(error.subject(), Some(&name("db")));
}

#[test]
fn several_unknown_prerequisites_are_named_in_the_plural() {
    let error = DomainError::UnknownPrerequisites {
        dependency: name("db"),
        missing: vec![name("zeta"), name("alpha")],
        available: vec![name("config")],
    };

    assert_eq!(
        error.to_string(),
        "unknown dependencies 'zeta, alpha' on field 'db'. You must choose between the dependencies the container can wire: [config]"
    );
}

#[test]
fn a_cycle_points_at_where_it_starts() {
    assert_eq!(cycle("a", &["b"]).subject(), Some(&name("a")));
}

#[test]
fn an_error_about_the_whole_container_has_no_subject() {
    let error = DomainError::TupleStruct;

    assert_eq!(error.subject(), None);
}

#[test]
fn a_trait_object_is_told_the_factory_method_named_after_its_field() {
    let error = DomainError::TraitObject {
        dependency: name("cache"),
        problem: TraitObjectProblem::WithoutFactory,
    };

    assert_eq!(
        error.to_string(),
        "field 'cache' has a trait-object type, which has no constructor; add build = factory to the field and build it in the build_cache method of the container's factories trait"
    );
}
