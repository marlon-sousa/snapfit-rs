Feature: Container Dependency Resolution

  As a Rust developer building services with snapfit
  I want container-managed singletons to resolve topologically and invalid graphs to be rejected at compile time
  So that dependent components receive shared instances without domain pollution or runtime wiring errors

  @v0_1_0
  Scenario: Singletons are instantiated once and shared by reference
    Given an AppContainer with leaf Config, Database, and UserService
    When the container is instantiated
    Then the container should provide a Config singleton
    And the container should provide a Database singleton
    And the container should provide a UserService singleton
    And the Database singleton should reference the exact same Config instance as the container
    And the UserService singleton should reference the exact same Database instance as the container

  @v0_1_0
  Scenario: Multiple dependencies of the same type are distinguished by field name
    Given an AppContainer with "read_db" and "write_db" singletons of type Database
    When the container is instantiated
    Then the container should provide separate "read_db" and "write_db" instances
    And the "read_db" instance should have a different memory address from the "write_db" instance

  @v0_1_0
  Scenario Outline: Invalid container declarations are refused at compile time with actionable diagnostics
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error>"

    Examples:
      | fixture                  | expected_error                                                                                                                     |
      | unknown_dependency       | unknown dependency 'confog' on field 'db'. You must choose between the dependencies the container can wire: [config, user_service] |
      | dependency_cycle         | dependency cycle detected: service_a -> service_b -> service_a                                                                     |
      | self_dependency_cycle    | dependency cycle detected: service_a -> service_a                                                                                  |
      | conflicting_lifecycle    | field 'config' cannot declare multiple lifecycle attributes                                                                        |
      | tuple_struct_container   | tuple structs are not supported by #[container]; use named fields                                                                  |
      | invalid_container_mode   | invalid container mode 'threaded': expected 'sync' or 'async'                                                                      |
      | sync_mode_with_async_dep | container with mode = "sync" cannot contain async dependencies                                                                     |
      | derive_with_transient    | #[derive(Container)] does not support #[transient] or #[dispatch] fields; use #[container] instead                                 |

  @v0_1_0
  Scenario Outline: APIs that are not stable in 0.1.0 require the unstable feature
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error> is unstable in this version of snapfit; enable the `unstable` feature of the `snapfit` crate to use it"

    Examples:
      | fixture                   | expected_error                                    |
      | unstable_async_container  | #[container(mode = "async")]                      |
      | unstable_async_dependency | async dependency 'db'                             |
      | unstable_transient        | transient dependency 'cart'                       |
      | unstable_builder          | custom builder 'build_auth' on dependency 'auth'  |

  @v0_1_0
  Scenario Outline: Planned container features are refused until they are implemented
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error> is planned for a future version of snapfit and is not implemented yet"

    Examples:
      | fixture                  | expected_error                          |
      | planned_parent_container | #[container(parent = ...)]              |
      | planned_introspection    | #[container(introspection = true)]      |
      | planned_dispatch         | #[dispatch(...)] on dependency 'store'  |
