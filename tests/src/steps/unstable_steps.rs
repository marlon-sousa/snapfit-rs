use std::sync::Arc;

use cucumber::{given, then, when};

use crate::world::{ContainerWorld, ScenarioBlueprint};

#[given("an async container whose gateway connects asynchronously")]
fn given_async_container(world: &mut ContainerWorld) {
    world.select(ScenarioBlueprint::AsyncShop);
}

#[when("the async container is initialized")]
async fn when_async_container_initialized(world: &mut ContainerWorld) {
    world.initialize_async_selected().await;
}

#[then("the order service should use the connected gateway")]
fn then_orders_use_connected_gateway(world: &mut ContainerWorld) {
    let container = world.unstable().async_shop();
    let gateway = container.get_gateway();

    assert_eq!(gateway.connected_to, "Northwind");
    assert!(
        Arc::ptr_eq(&gateway, &container.get_orders().gateway_ref()),
        "Expected OrderService to hold the container's gateway singleton"
    );
}

#[then("the gateway getter should return the same singleton every time")]
fn then_gateway_is_singleton(world: &mut ContainerWorld) {
    let container = world.unstable().async_shop();
    assert!(Arc::ptr_eq(
        &container.get_gateway(),
        &container.get_gateway()
    ));
}

#[given("a container with a transient cart that depends on the shop config")]
fn given_transient_container(world: &mut ContainerWorld) {
    world.select(ScenarioBlueprint::TransientShop);
}

#[then("two carts should be different instances")]
fn then_carts_differ(world: &mut ContainerWorld) {
    let container = world.unstable().transient_shop();
    let first = container.get_cart();
    let second = container.get_cart();
    assert_ne!(
        first.id, second.id,
        "Expected a fresh Cart on every get_cart()"
    );
}

#[then("both carts should reference the container's shop config")]
fn then_carts_share_config(world: &mut ContainerWorld) {
    let container = world.unstable().transient_shop();
    let config = container.get_config();
    assert!(Arc::ptr_eq(&config, &container.get_cart().config_ref()));
    assert!(Arc::ptr_eq(&config, &container.get_cart().config_ref()));
}

#[given(expr = "a container whose auth service is made by the builder {string}")]
fn given_builder_container(world: &mut ContainerWorld, builder: String) {
    assert_eq!(builder, "build_auth");
    world.select(ScenarioBlueprint::BuilderShop);
}

#[then("the auth service should be configured by the builder with the shop config")]
fn then_auth_built_by_builder(world: &mut ContainerWorld) {
    let container = world.unstable().builder_shop();
    assert_eq!(container.get_auth().realm, "Northwind-auth");
}
