use std::sync::Arc;

use cucumber::{given, then, when};

use crate::world::{ContainerWorld, ScenarioBlueprint};

#[given("an AppContainer with leaf Config, Database, and UserService")]
fn given_shared_singleton_container(world: &mut ContainerWorld) {
    world.select(ScenarioBlueprint::SharedSingletons);
}

#[given("an AppContainer with \"read_db\" and \"write_db\" singletons of type Database")]
fn given_multi_db_container(world: &mut ContainerWorld) {
    world.select(ScenarioBlueprint::MultiDatabase);
}

#[when("the container is instantiated")]
fn when_container_instantiated(world: &mut ContainerWorld) {
    world.instantiate_selected();
}

#[then("the container should provide a Config singleton")]
fn then_provides_config_singleton(world: &mut ContainerWorld) {
    let container = world.app_container();
    let first = container.get_config();
    let second = container.get_config();

    assert_eq!(first.app_name, "snapfit-mvp");
    assert!(
        Arc::ptr_eq(&first, &second),
        "Expected get_config() to return the same Config singleton instance"
    );
}

#[then("the container should provide a Database singleton")]
fn then_provides_database_singleton(world: &mut ContainerWorld) {
    let container = world.app_container();
    let first = container.get_db();
    let second = container.get_db();

    assert!(
        Arc::ptr_eq(&first, &second),
        "Expected get_db() to return the same Database singleton instance"
    );
}

#[then("the container should provide a UserService singleton")]
fn then_provides_user_service_singleton(world: &mut ContainerWorld) {
    let container = world.app_container();
    let first = container.get_user_service();
    let second = container.get_user_service();

    assert!(
        Arc::ptr_eq(&first, &second),
        "Expected get_user_service() to return the same UserService singleton instance"
    );
}

#[then("the Database singleton should reference the exact same Config instance as the container")]
fn then_db_references_same_config(world: &mut ContainerWorld) {
    let container = world.app_container();
    let container_config = container.get_config();
    let db_config = container.get_db().get_config_ref();

    assert!(
        Arc::ptr_eq(&container_config, &db_config),
        "Expected Database's Config pointer to match Container's Config pointer"
    );
}

#[then(
    "the UserService singleton should reference the exact same Database instance as the container"
)]
fn then_user_service_references_same_db(world: &mut ContainerWorld) {
    let container = world.app_container();
    let container_db = container.get_db();
    let service_db = container.get_user_service().get_db_ref();

    assert!(
        Arc::ptr_eq(&container_db, &service_db),
        "Expected UserService's Database pointer to match Container's Database pointer"
    );
}

#[then("the container should provide separate \"read_db\" and \"write_db\" instances")]
fn then_provides_separate_read_and_write_db(world: &mut ContainerWorld) {
    let container = world.multi_db_container();
    let read_first = container.get_read_db();
    let read_second = container.get_read_db();
    let write_first = container.get_write_db();
    let write_second = container.get_write_db();

    assert!(
        Arc::ptr_eq(&read_first, &read_second),
        "Expected get_read_db() to consistently return the read_db singleton"
    );
    assert!(
        Arc::ptr_eq(&write_first, &write_second),
        "Expected get_write_db() to consistently return the write_db singleton"
    );
}

#[then(
    "the \"read_db\" instance should have a different memory address from the \"write_db\" instance"
)]
fn then_read_and_write_db_have_different_addresses(world: &mut ContainerWorld) {
    let container = world.multi_db_container();
    let read_db = container.get_read_db();
    let write_db = container.get_write_db();

    assert!(
        !Arc::ptr_eq(&read_db, &write_db),
        "Expected read_db and write_db to be distinct singleton instances"
    );
    assert_ne!(
        Arc::as_ptr(&read_db),
        Arc::as_ptr(&write_db),
        "Expected read_db and write_db to have different memory addresses"
    );
}
