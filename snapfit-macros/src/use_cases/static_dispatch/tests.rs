use super::*;
use std::iter;

use crate::domain::{
    AttributePosition, ContainerName, DependencyName, DomainError, StaticDispatchProblem,
};

type Container = ContainerDeclaration<(), (), &'static str, &'static str>;

mod violations;

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

#[test]
fn a_refused_container_falls_back_to_what_its_readable_parts_give_each_type() {
    let fallbacks = fallback_dispatched_types(
        &[
            declared("Db", 0, &["Fake"]),
            declared("Cache", 1, &[]),
            declared("Db", 2, &[]),
        ],
        &[extended("Db", 3, &["Mock"])],
        &[field("main", named("Db"), Some("MySql"))],
    );

    assert_eq!(
        fallbacks,
        [
            DispatchedTypeFallback::Planned(DispatchedTypePlan::new(
                "Db",
                variant("MySql"),
                vec![variant("Fake"), variant("Mock")],
            )),
            DispatchedTypeFallback::Unplanned("Cache"),
        ]
    );
}
