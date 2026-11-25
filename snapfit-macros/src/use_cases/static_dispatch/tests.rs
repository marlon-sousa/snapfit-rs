use super::*;
use crate::domain::{AttributePosition, ContainerName, DependencyName};

type Container = ContainerDeclaration<(), (), &'static str, &'static str>;

fn type_name(text: &str) -> TypeName {
    TypeName::new(text).unwrap()
}

fn variant(text: &'static str) -> Variant<&'static str> {
    Variant::new(type_name(text), text)
}

fn declared(
    name: &'static str,
    position: usize,
    extra: &[&'static str],
) -> DispatchedTypeDeclaration<&'static str, &'static str> {
    let mut declaration =
        DispatchedTypeDeclaration::new(type_name(name), AttributePosition(position), name);
    declaration.extra = extra.iter().copied().map(variant).collect();
    declaration
}

fn extended(
    name: &str,
    position: usize,
    variants: &[&'static str],
) -> DispatchExtension<&'static str> {
    DispatchExtension::new(
        type_name(name),
        AttributePosition(position),
        variants.iter().copied().map(variant).collect(),
    )
}

/// A field named `field` whose value is written `written`, naming `implementation` if any.
fn field(
    field: &str,
    written: WrittenType,
    implementation: Option<&'static str>,
) -> DependencyDeclaration<(), &'static str> {
    let mut dependency = DependencyDeclaration::new(DependencyName::new(field).unwrap(), ());
    dependency.written_type = written;
    dependency.implementation = implementation.map_or(Implementation::OfField, |text| {
        Implementation::Named(variant(text))
    });
    dependency
}

fn container(
    dependencies: Vec<DependencyDeclaration<(), &'static str>>,
    dispatched_types: Vec<DispatchedTypeDeclaration<&'static str, &'static str>>,
    extensions: Vec<DispatchExtension<&'static str>>,
) -> Container {
    let mut container = ContainerDeclaration::new(ContainerName::new("App").unwrap(), ());
    container.dependencies = dependencies;
    container.dispatched_types = dispatched_types;
    container.extensions = extensions;
    container
}

fn named(text: &str) -> WrittenType {
    WrittenType::Name(type_name(text))
}

fn plan_names(plan: &DispatchedTypePlan<&'static str, &'static str>) -> Vec<&'static str> {
    iter::once(&plan.first)
        .chain(&plan.rest)
        .map(|variant| variant.syntax)
        .collect()
}

#[test]
fn a_dispatched_type_holds_its_fields_then_its_extra_then_its_extensions_each_once() {
    let container = container(
        vec![
            field("main", named("Db"), Some("MySql")),
            field("other", named("Cache"), Some("Redis")),
            field("backup", named("Db"), Some("Sqlite")),
            field("again", named("Db"), Some("MySql")),
        ],
        vec![declared("Db", 0, &["Fake", "Sqlite"])],
        vec![
            extended("Db", 1, &["Mock", "Fake"]),
            extended("Cache", 2, &["Lost"]),
        ],
    );

    let plans = dispatched_types(&container);

    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].syntax, "Db");
    assert_eq!(plan_names(&plans[0]), ["MySql", "Sqlite", "Fake", "Mock"]);
}

#[test]
fn a_repeated_declaration_is_planned_once_under_the_first() {
    let container = container(
        Vec::new(),
        vec![
            declared("Db", 0, &["MySql"]),
            declared("Db", 1, &["Sqlite"]),
        ],
        Vec::new(),
    );

    let plans = dispatched_types(&container);

    assert_eq!(plans.len(), 1);
    assert_eq!(plan_names(&plans[0]), ["MySql"]);
}

#[test]
fn a_dispatched_type_without_implementations_is_not_planned_and_is_refused() {
    let container = container(Vec::new(), vec![declared("Db", 3, &[])], Vec::new());

    assert!(dispatched_types(&container).is_empty());
    assert_eq!(
        dispatch_violations(&container),
        [DomainError::StaticDispatch(
            StaticDispatchProblem::WithoutImplementations {
                position: AttributePosition(3),
                name: type_name("Db"),
            }
        )]
    );
}

#[test]
fn every_problem_of_the_declarations_is_reported_together() {
    let container = container(
        vec![field(
            "main",
            WrittenType::Path {
                last: type_name("Db"),
                written: "shop::Db".to_owned(),
            },
            Some("MySql"),
        )],
        vec![
            declared("Db", 0, &["MySql"]),
            declared("Db", 1, &["Sqlite"]),
            declared("Cache", 2, &[]),
        ],
        vec![extended("Queue", 3, &["Kafka"])],
    );

    assert_eq!(
        dispatch_violations(&container),
        [
            StaticDispatchProblem::RepeatedName {
                position: AttributePosition(1),
                name: type_name("Db")
            },
            StaticDispatchProblem::Undeclared {
                position: AttributePosition(3),
                name: type_name("Queue"),
                declared: vec![type_name("Db"), type_name("Cache")],
            },
            StaticDispatchProblem::WithoutImplementations {
                position: AttributePosition(2),
                name: type_name("Cache")
            },
            StaticDispatchProblem::ThroughPath {
                dependency: DependencyName::new("main").unwrap(),
                name: type_name("Db"),
                written: "shop::Db".to_owned(),
            },
        ]
        .map(DomainError::StaticDispatch)
    );
}

#[test]
fn a_field_holds_a_declared_name_written_alone_or_through_a_path() {
    let names: DispatchedNames = [declared("Db", 0, &[])].into_iter().collect();
    let path = WrittenType::Path {
        last: type_name("Db"),
        written: "shop::Db".to_owned(),
    };

    assert_eq!(
        names.held_by(&field("a", named("Db"), None)),
        Some(&type_name("Db"))
    );
    assert_eq!(
        names.held_by(&field("b", path, None)),
        Some(&type_name("Db"))
    );
    assert_eq!(names.held_by(&field("c", named("Cache"), None)), None);
    assert_eq!(names.held_by(&field("d", WrittenType::Other, None)), None);
    assert_eq!(dispatch_of(Some(&type_name("Db"))), Dispatch::Static);
    assert_eq!(dispatch_of(None), Dispatch::None);
}
