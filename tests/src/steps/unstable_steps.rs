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

#[given("a container whose transient auth session is built by a factory from the shop config")]
fn given_transient_factory_container(world: &mut ContainerWorld) {
    world.select(ScenarioBlueprint::TransientFactoryShop);
}

#[then("every auth session should be built again by the factory from the container's shop config")]
fn then_sessions_built_by_factory(world: &mut ContainerWorld) {
    let container = world.unstable().transient_factory_shop();
    let first = container.get_session();
    let second = container.get_session();
    assert_ne!(
        first.id, second.id,
        "Expected a fresh AuthSession on every get"
    );
    assert_eq!(first.realm, "Northwind-auth");
    assert!(Arc::ptr_eq(&first.config, &container.get_config()));
    assert!(Arc::ptr_eq(&second.config, &container.get_config()));
}

#[given("a container where a transient price list feeds a transient quote and a singleton catalog")]
fn given_layered_container(world: &mut ContainerWorld) {
    world.select(ScenarioBlueprint::LayeredShop);
}

#[then("every quote should get its own fresh price list")]
fn then_quotes_get_fresh_price_lists(world: &mut ContainerWorld) {
    let container = world.unstable().layered_shop();
    let first = container.get_quote().prices.id;
    let second = container.get_quote().prices.id;
    assert_ne!(
        first, second,
        "Expected a fresh PriceList inside every Quote"
    );
}

#[then("the catalog should keep the price list it was built with")]
fn then_catalog_keeps_its_price_list(world: &mut ContainerWorld) {
    let container = world.unstable().layered_shop();
    let catalog = container.get_catalog();
    assert!(Arc::ptr_eq(&catalog, &container.get_catalog()));
    assert_ne!(catalog.prices.id, container.get_prices().id);
}

#[then(expr = "the trait-object notifier made by its factory should send through {string}")]
fn then_notifier_uses_channel(world: &mut ContainerWorld, channel: String) {
    let container = world.unstable().layered_shop();
    assert_eq!(container.get_notifier().channel(), channel);
}

#[when("the async depot whose leaf_build is default is initialized")]
async fn when_depot_initialized(world: &mut ContainerWorld) {
    world.initialize_depot().await;
}

#[then("its region should be built by Default, and its async warehouse by its own new")]
fn then_async_leaf_keeps_new(world: &mut ContainerWorld) {
    let depot = world.unstable().depot();

    assert_eq!(depot.get_region().code, "");
    assert_eq!(depot.get_warehouse().opened_by, "new");
}
