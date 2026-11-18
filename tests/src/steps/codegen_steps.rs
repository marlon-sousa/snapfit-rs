use std::collections::BTreeMap;
use std::sync::Arc;

use cucumber::{given, then};

use crate::world::{ContainerWorld, ScenarioBlueprint};

#[given("a container whose fields use plain, fully qualified and raw-identifier types")]
fn given_typed_fields_container(world: &mut ContainerWorld) {
    world.select(ScenarioBlueprint::TypedFields);
}

#[then("the plain fields should be built with their own constructors")]
fn then_plain_fields_use_their_constructors(world: &mut ContainerWorld) {
    let container = world.typed_fields_container();
    assert_eq!(container.get_tags(), Vec::<String>::new());
    assert_eq!(container.get_type(), BTreeMap::new());
}

#[then("the database should hold the container's fully qualified config singleton")]
fn then_db_holds_qualified_config(world: &mut ContainerWorld) {
    let container = world.typed_fields_container();
    assert!(
        Arc::ptr_eq(
            &container.get_config(),
            &container.get_db().get_config_ref()
        ),
        "Expected Database to receive the container's ::std::sync::Arc<Config> singleton"
    );
}

#[then("the field whose deps write config as r#config should hold that same config singleton")]
fn then_raw_dependency_names_the_field(world: &mut ContainerWorld) {
    let container = world.typed_fields_container();
    assert!(
        Arc::ptr_eq(
            &container.get_config(),
            &container.get_ref().get_config_ref()
        ),
        "Expected deps = [r#config] to wire the container's config field"
    );
}

#[given(regex = r"^a container that derives Clone and Debug (before|after) #\[container\]$")]
fn given_derived_container(world: &mut ContainerWorld, placement: String) {
    world.select(match placement.as_str() {
        "before" => ScenarioBlueprint::DerivesBeforeAttribute,
        _ => ScenarioBlueprint::DerivesAfterAttribute,
    });
}

#[then("a clone of the container should share its singletons")]
fn then_clone_shares_singletons(world: &mut ContainerWorld) {
    let container = world.derived_container();
    assert!(
        Arc::ptr_eq(&container.db(), &container.db_of_clone()),
        "Expected the derived Clone to copy the Arc, not rebuild the singleton"
    );
}

#[then(expr = "its Debug output should start with {string}")]
fn then_debug_output_starts_with(world: &mut ContainerWorld, prefix: String) {
    let text = world.derived_container().debug_text();
    assert!(
        text.starts_with(&prefix),
        "Expected Debug output to start with '{prefix}', got: {text}"
    );
}

#[given("a container whose non-snake-case field is named like a unit struct in scope")]
fn given_unit_struct_named_field(world: &mut ContainerWorld) {
    world.select(ScenarioBlueprint::UnitStructNamedField);
}

#[then("that field's getter should return the same singleton every time")]
fn then_unit_struct_named_field_is_singleton(world: &mut ContainerWorld) {
    let container = world.unit_struct_named_field();
    assert!(Arc::ptr_eq(
        &container.get_Marker(),
        &container.get_Marker()
    ));
}
