Feature: Unstable lifecycles behind the `unstable` feature

  As a Rust developer who opted in to snapfit's unstable APIs
  I want async containers and transient dependencies to keep working
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
  Scenario: A transient factory builds a fresh value on every get from the container's singletons
    Given a container whose transient auth session is built by a factory from the shop config
    When the container is instantiated
    Then every auth session should be built again by the factory from the container's shop config

  @unstable @regression
  Scenario: Transients can feed other transients and singletons, and factories can make trait objects
    Given a container where a transient price list feeds a transient quote and a singleton catalog
    When the container is instantiated
    Then every quote should get its own fresh price list
    And the catalog should keep the price list it was built with
    And the trait-object notifier made by its factory should send through "email:northwind"

  @unstable
  Scenario: A container's leaf_build skips its async leaves, which keep their own new
    When the async depot whose leaf_build is default is initialized
    Then its region should be built by Default, and its async warehouse by its own new

  @unstable
  Scenario: An async field cannot be built by Default
    Given an invalid container fixture "async_default"
    When the fixture is compiled
    Then compilation should fail with "async dependency 'db' cannot be built with build = default, because Default::default() cannot be awaited; remove async, or build it with build = constructor and the name of an async function in brackets"
    And the error should point at line 9 of the fixture

  @unstable
  Scenario Outline: Planned container features stay refused with the unstable feature enabled
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error> is planned for a future version of snapfit and is not implemented yet"
    And the error should point at line <line> of the fixture

    Examples:
      | fixture                  | line | expected_error                          |
      | planned_parent_container | 7    | #[container(parent = ...)]              |
      | planned_introspection    | 7    | #[container(introspection = true)]      |
      | planned_dispatch         | 10   | #[dispatch(...)] on dependency 'store'  |
      | generic_container        | 4    | generic parameters on container 'Repo'  |
      | planned_async_factory    | 8    | build = factory on async dependency 'db' |
      | planned_async_builder    | 8    | build = builder on async dependency 'exporter' |

  @unstable @regression
  Scenario: A trait-object field without a factory is told to build it in one, as on the stable build
    Given an invalid container fixture "trait_object_without_factory"
    When the fixture is compiled
    Then compilation should fail with "field 'store' has a trait-object type, which has no constructor; add build = factory to the field and build it in the build_store method of the container's factories trait"
    And the error should point at line 8 of the fixture
