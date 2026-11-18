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
  Scenario: Values are built in declaration order, each after the fields it needs
    When the container declaring service, db, config and clock in that order is instantiated
    Then its values should be built in the order "config, db, service, clock"

  @v0_1_0
  Scenario: A field holds the concrete type a constructor takes as a trait object
    When the container whose payments field holds the concrete StripeGateway is instantiated
    Then the checkout service, whose constructor takes the PaymentGateway trait object, should charge through that same gateway

  @v0_1_0
  Scenario Outline: Invalid container declarations are refused at compile time with actionable diagnostics
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error>"
    And the error should point at line <line> of the fixture

    Examples:
      | fixture                    | line | expected_error                                                                                                                     |
      | unknown_dependency         | 13   | unknown dependency 'confog' on field 'db'. You must choose between the dependencies the container can wire: [config, user_service] |
      | dependency_cycle           | 9    | dependency cycle: service_a depends on service_b, which depends on service_a                                                       |
      | self_dependency_cycle      | 8    | dependency cycle: service_a depends on itself                                                                                      |
      | conflicting_lifecycle      | 8    | field 'config' cannot declare multiple lifecycle attributes                                                                        |
      | tuple_struct_container     | 7    | tuple structs are not supported by #[container]; use named fields                                                                  |
      | second_container_attribute | 8    | multiple #[container] attributes are not allowed                                                                                   |
      | sync_mode_with_async_dep   | 8    | container with mode = "sync" cannot contain async dependencies                                                                     |

  @v0_1_0
  Scenario: A container is declared with the attribute, never with a derive
    Given an invalid container fixture "derive_container"
    When the fixture is compiled
    Then compilation should fail with "cannot find derive macro `Container` in this scope"
    And the error should point at line 7 of the fixture

  @v0_1_0 @regression
  Scenario: A field without a lifecycle is offered only what the stable build accepts
    Given an invalid container fixture "several_field_problems"
    When the fixture is compiled
    Then compilation should fail with "field 'config' has no snapfit attribute; add #[singleton] to have the container build it"
    And the error should point at line 9 of the fixture
    And the compiler output should not mention "#[transient]"
    And the compiler output should not mention "#[dispatch]"

  @v0_1_0 @regression
  Scenario: An unknown container mode is offered only what the stable build accepts
    Given an invalid container fixture "invalid_container_mode"
    When the fixture is compiled
    Then compilation should fail with "invalid container mode 'threaded': write mode = "sync", or leave mode out"
    And the error should point at line 7 of the fixture
    And the compiler output should not mention "async"

  @v0_1_0
  Scenario: Every problem in the dependency graph is reported in one compile, each at its field
    Given an invalid container fixture "several_graph_problems"
    When the fixture is compiled
    Then compilation should fail with "dependency cycle: service_a depends on service_b, which depends on service_a"
    And compilation should fail with "unknown dependency 'confog' on field 'db'"
    And compilation should fail with "field 'store' has a trait-object type, which has no constructor"
    And the compiler should report 3 errors
    And the error should point at line 11 of the fixture
    And the error should point at line 17 of the fixture
    And the error should point at line 20 of the fixture

  @v0_1_0
  Scenario: Every badly declared field is reported in one compile
    Given an invalid container fixture "several_field_problems"
    When the fixture is compiled
    Then compilation should fail with "field 'config' has no snapfit attribute; add #[singleton] to have the container build it"
    And compilation should fail with "constructor "connect" on field 'db' must be written without quotes: build = constructor(connect)"
    And the compiler should report 2 errors
    And the error should point at line 9 of the fixture
    And the error should point at line 11 of the fixture

  @v0_1_0 @regression
  Scenario: Every bad entry of deps is reported in one compile, in words, beside the attribute's other problems
    Given an invalid container fixture "several_bad_deps"
    When the fixture is compiled
    Then compilation should fail with "field 'cache': the deps entry 'config + 1' is not the name of a field; write another field's name, such as config"
    And compilation should fail with "field 'cache': the deps entry 'a::b = config' is not a label; write the setter, then '=', then the field, each a single name, such as disk = folder_files"
    And compilation should fail with "field 'mailer': 'deps = config' is not a list of fields; write the fields in square brackets, such as deps = [config]"
    And compilation should fail with "field 'clock': Unknown field: `bogus`"
    And compilation should fail with "field 'clock': the deps entry 'parent()' names no single dependency; parent(...) is planned for a future version of snapfit and is not implemented yet"
    And the compiler should report 5 errors
    And the error should point at line 14 of the fixture
    And the error should point at line 17 of the fixture
    And the error should point at line 20 of the fixture

  @v0_1_0 @regression
  Scenario: Each field listed more than once in deps is refused once
    Given an invalid container fixture "repeated_deps"
    When the fixture is compiled
    Then compilation should fail with "field 'cache' lists 'config' in deps more than once; list each dependency once"
    And compilation should fail with "field 'cache' lists 'store' in deps more than once; list each dependency once"
    And the compiler should report 2 errors
    And the error should point at line 16 of the fixture

  @v0_1_0 @regression
  Scenario: An unknown dependency listed twice is named once
    Given an invalid container fixture "unknown_dependency_listed_twice"
    When the fixture is compiled
    Then compilation should fail with "unknown dependency 'nosuch' on field 'cache'. You must choose between the dependencies the container can wire: [config, pool]"
    And compilation should fail with "field 'cache' lists 'nosuch' in deps more than once; list each dependency once"
    And compilation should fail with "unknown dependency 'nosuch' on field 'pool'. You must choose between the dependencies the container can wire: [config, cache]"
    And the compiler should report 3 errors
    And the error should point at line 18 of the fixture
    And the error should point at line 21 of the fixture

  @v0_1_0 @regression
  Scenario: Several unknown dependencies of one field are named in the plural
    Given an invalid container fixture "several_unknown_dependencies"
    When the fixture is compiled
    Then compilation should fail with "unknown dependencies 'zeta, alpha' on field 'db'. You must choose between the dependencies the container can wire: [config]"
    And the error should point at line 12 of the fixture

  @v0_1_0 @regression
  Scenario: deps written in parentheses is refused, saying to write square brackets
    Given an invalid container fixture "parenthesised_deps"
    When the fixture is compiled
    Then compilation should fail with "field 'mailer': 'deps(config)' writes its fields in parentheses; write them in square brackets: deps = [config]"
    And the error should point at line 12 of the fixture

  @v0_1_0
  Scenario Outline: APIs that are not stable in 0.1.0 require the unstable feature
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error> is unstable in this version of snapfit; enable the `unstable` feature of the `snapfit` crate to use it"
    And the error should point at line <line> of the fixture

    Examples:
      | fixture                   | line | expected_error                                    |
      | unstable_async_container  | 7    | #[container(mode = "async")]                      |
      | unstable_async_dependency | 8    | async dependency 'db'                             |
      | unstable_transient        | 12   | transient dependency 'cart'                       |

  @v0_1_0
  Scenario Outline: Planned container features are refused until they are implemented
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
