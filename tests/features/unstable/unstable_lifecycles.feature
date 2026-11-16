Feature: Unstable lifecycles behind the `unstable` feature

  As a Rust developer who opted in to snapfit's unstable APIs
  I want async containers, transient dependencies and custom builders to keep working
  So that I can try them before their API is stabilised

  @unstable
  Scenario: An async container awaits its async singletons once at startup
    Given an async container whose gateway connects asynchronously
    When the async container is initialized
    Then the order service should use the connected gateway
    And the gateway getter should return the same singleton every time

  @unstable
  Scenario: A transient dependency is built fresh on every get and shares its singletons
    Given a container with a transient cart that depends on the shop config
    When the container is instantiated
    Then two carts should be different instances
    And both carts should reference the container's shop config

  @unstable
  Scenario: A custom builder on the container constructs its dependency
    Given a container whose auth service is made by the builder "build_auth"
    When the container is instantiated
    Then the auth service should be configured by the builder with the shop config

  @unstable @regression
  Scenario: Transients can feed other transients and singletons, and builders can make trait objects
    Given a container where a transient price list feeds a transient quote and a singleton catalog
    When the container is instantiated
    Then every quote should get its own fresh price list
    And the catalog should keep the price list it was built with
    And the trait-object notifier made by its builder should send through "email:northwind"

  @unstable
  Scenario Outline: Planned container features stay refused with the unstable feature enabled
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error> is planned for a future version of snapfit and is not implemented yet"

    Examples:
      | fixture                  | expected_error                          |
      | planned_parent_container | #[container(parent = ...)]              |
      | planned_introspection    | #[container(introspection = true)]      |
      | planned_dispatch         | #[dispatch(...)] on dependency 'store'  |
      | generic_container        | generic parameters on container 'Repo'  |
