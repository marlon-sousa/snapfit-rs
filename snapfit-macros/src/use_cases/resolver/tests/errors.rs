use super::*;

#[test]
fn every_unknown_prerequisite_is_reported_with_what_could_be_wired() {
    let errors = errors_of(vec![
        dependency("config", &[]),
        dependency("db", &["confog", "config", "pool"]),
        dependency("cache", &["redis"]),
    ]);

    assert_eq!(
        errors,
        [
            DomainError::Graph(GraphProblem::UnknownPrerequisites {
                dependency: name("db"),
                missing: names(&["confog", "pool"]),
                available: names(&["config", "cache"]),
            }),
            DomainError::Graph(GraphProblem::UnknownPrerequisites {
                dependency: name("cache"),
                missing: names(&["redis"]),
                available: names(&["config", "db"]),
            }),
        ]
    );
}

#[test]
fn an_unknown_prerequisite_listed_twice_is_named_once() {
    let errors = errors_of(vec![
        dependency("config", &[]),
        dependency("db", &["nosuch", "config", "pool", "nosuch", "pool"]),
    ]);

    assert_eq!(
        errors,
        [DomainError::Graph(GraphProblem::UnknownPrerequisites {
            dependency: name("db"),
            missing: names(&["nosuch", "pool"]),
            available: names(&["config"]),
        })]
    );
}

#[test]
fn a_cycle_is_reported_from_the_first_field_on_it() {
    let errors = errors_of(vec![
        dependency("service_a", &["service_b"]),
        dependency("service_b", &["service_c"]),
        dependency("service_c", &["service_a"]),
    ]);

    assert_eq!(
        errors,
        [DomainError::Graph(GraphProblem::Cycle {
            start: name("service_a"),
            through: names(&["service_b", "service_c"]),
        })]
    );
}

#[test]
fn a_dependency_that_needs_itself_is_a_cycle_of_one() {
    let errors = errors_of(vec![dependency("service_a", &["service_a"])]);

    assert_eq!(
        errors,
        [DomainError::Graph(GraphProblem::Cycle {
            start: name("service_a"),
            through: Vec::new(),
        })]
    );
}

#[test]
fn separate_cycles_are_each_reported() {
    let errors = errors_of(vec![
        dependency("a", &["b"]),
        dependency("b", &["a"]),
        dependency("c", &["d"]),
        dependency("d", &["c"]),
    ]);

    let starts: Vec<_> = errors.iter().filter_map(DomainError::subject).collect();
    assert_eq!(starts, [&name("a"), &name("c")]);
}

#[test]
fn a_dependency_on_a_cycle_is_not_reported_again() {
    let errors = errors_of(vec![
        dependency("a", &["b"]),
        dependency("b", &["a"]),
        dependency("user", &["a"]),
    ]);

    assert_eq!(errors.len(), 1);
}

#[test]
fn a_cycle_and_an_unknown_prerequisite_are_both_reported() {
    let errors = errors_of(vec![
        dependency("a", &["b"]),
        dependency("b", &["a"]),
        dependency("c", &["nowhere"]),
    ]);

    assert!(matches!(
        errors[0],
        DomainError::Graph(GraphProblem::Cycle { .. })
    ));
    assert!(matches!(
        errors[1],
        DomainError::Graph(GraphProblem::UnknownPrerequisites { .. })
    ));
}

#[test]
fn a_duplicate_name_is_reported_and_the_first_declaration_kept() {
    let errors = errors_of(vec![dependency("db", &[]), dependency("db", &["config"])]);

    assert_eq!(
        errors,
        [DomainError::Declaration(
            DeclarationProblem::DuplicateDependency(name("db"))
        )]
    );
}

#[test]
fn a_trait_object_without_a_factory_is_reported_beside_graph_errors() {
    let mut store = dependency("store", &[]);
    store.shape = TypeShape::TraitObject;

    let errors = errors_of(vec![store, dependency("a", &["a"])]);

    assert!(matches!(errors[0], DomainError::TraitObject { .. }));
    assert!(matches!(
        errors[1],
        DomainError::Graph(GraphProblem::Cycle { .. })
    ));
}

#[test]
fn a_cycle_entered_from_its_middle_is_told_from_its_first_declared_field() {
    let errors = errors_of(vec![
        dependency("x", &["b"]),
        dependency("a", &["b"]),
        dependency("b", &["a"]),
    ]);

    assert_eq!(
        errors,
        [DomainError::Graph(GraphProblem::Cycle {
            start: name("a"),
            through: names(&["b"]),
        })]
    );
}
