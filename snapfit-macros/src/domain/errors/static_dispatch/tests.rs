use super::*;

fn type_name(text: &str) -> TypeName {
    TypeName::new(text).unwrap()
}

fn undeclared_queue(declared: &[&str]) -> StaticDispatchProblem {
    StaticDispatchProblem::Undeclared {
        position: AttributePosition(1),
        name: type_name("Queue"),
        declared: declared.iter().copied().map(type_name).collect(),
    }
}

#[test]
fn an_extension_of_an_undeclared_name_lists_the_declared_names_or_says_to_declare_one() {
    assert_eq!(
        undeclared_queue(&[]).to_string(),
        "extend_static_dispatch extends 'Queue', which this container does not declare; declare it first, such as static_dispatch_for(Database as Queue)"
    );
    assert_eq!(
        undeclared_queue(&["Db", "Cache"]).to_string(),
        format!(
            "extend_static_dispatch extends 'Queue', which this container does not declare; extend one of the types its static_dispatch_for declares: [{}]",
            join_names(&[type_name("Db"), type_name("Cache")])
        )
    );
}

#[test]
fn an_unreadable_attribute_names_its_kind_and_shows_how_that_kind_is_written() {
    let unreadable = |kind| StaticDispatchProblem::Unreadable {
        position: AttributePosition(0),
        kind,
        problem: Unreadability::Syn("expected `as`".to_owned()),
    };

    assert_eq!(
        unreadable(DispatchAttributeKind::Declaration).to_string(),
        "static_dispatch_for cannot be read: expected `as`; write it as static_dispatch_for(Database as Db, extra = [Sqlite])"
    );
    assert_eq!(
        unreadable(DispatchAttributeKind::Extension).to_string(),
        "extend_static_dispatch cannot be read: expected `as`; write it as extend_static_dispatch(Db, [Mock])"
    );
}

#[test]
fn a_problem_points_at_its_attribute_or_at_its_field() {
    let field = DependencyName::new("r#type").unwrap();
    let without = StaticDispatchProblem::WithoutImplementation {
        dependency: field.clone(),
        name: type_name("Db"),
    };

    assert_eq!(undeclared_queue(&[]).position(), Some(AttributePosition(1)));
    assert_eq!(undeclared_queue(&[]).dependency(), None);
    assert_eq!(without.position(), None);
    assert_eq!(without.dependency(), Some(&field));
    assert!(
        without
            .to_string()
            .ends_with("in the build_type method of the container's factories trait")
    );
}
