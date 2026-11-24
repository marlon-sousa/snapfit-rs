Feature: Generated code follows the container as written

  As a Rust developer declaring a snapfit container
  I want the generated code to reuse my types, attributes and visibility exactly as I wrote them
  So that any valid Rust field type works and mistakes are reported on the line that caused them

  @v0_1_0 @regression
  Scenario: Fields of plain, fully qualified and raw-identifier types are wired
    Given a container whose fields use plain, fully qualified and raw-identifier types
    When the container is instantiated
    Then the plain fields should be built with their own constructors
    And the database should hold the container's fully qualified config singleton
    And the field whose deps write config as r#config should hold that same config singleton

  @v0_1_0 @regression
  Scenario: A field named like a unit struct in scope does not clash with the generated code
    Given a container whose non-snake-case field is named like a unit struct in scope
    When the container is instantiated
    Then that field's getter should return the same singleton every time

  @v0_1_0 @regression
  Scenario Outline: Derives on the container are kept whichever side of #[container] they are written
    Given a container that derives Clone and Debug <placement> #[container]
    When the container is instantiated
    Then a clone of the container should share its singletons
    And its Debug output should start with "<container> {"

    Examples:
      | placement | container              |
      | before    | DerivedBeforeContainer |
      | after     | DerivedAfterContainer  |

  @v0_1_0 @regression
  Scenario: The container keeps the visibility it was declared with
    Given an invalid container fixture "private_container"
    When the fixture is compiled
    Then compilation should fail with "struct `HiddenContainer` is private"

  @v0_1_0 @regression
  Scenario Outline: Fields snapfit cannot generate code for are refused with a clear error
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error>"
    And the error should point at line <line> of the fixture

    Examples:
      | fixture                  | line | expected_error                                                                               |
      | conditional_field        | 10   | field 'metrics' cannot use #[cfg] or #[cfg_attr]; conditional dependencies are not supported |
      | invalid_constructor_name | 12   | constructor 'connect()' on field 'db' must be a function name, such as build = constructor(connect) |
      | quoted_constructor_name  | 8    | constructor "connect" on field 'db' must be written without quotes: build = constructor(connect)    |

  @v0_1_0 @regression
  Scenario: A trait-object field outside an Arc, without a factory, is told to build it in one
    Given an invalid container fixture "trait_object_named_cache"
    When the fixture is compiled
    Then compilation should fail with "field 'cache' has a trait-object type, which has no constructor; add build = factory to the field and build it in the build_cache method of the container's factories trait"
    And the compiler output should not mention "builder ="
    And the error should point at line 7 of the fixture

  @v0_2_0
  Scenario: An Arc of a trait object without an implementation or a factory is told to name one or the other
    Given an invalid container fixture "trait_object_without_factory"
    When the fixture is compiled
    Then compilation should fail with "field 'store' has a trait-object type, which has no constructor; name the type the container builds, such as implementation = StripeGateway, or add build = factory to the field and build it in the build_store method of the container's factories trait"
    And the compiler output should not mention "builder ="
    And the error should point at line 8 of the fixture

  @v0_3_0
  Scenario Outline: A type that has the trait object as its own type argument, inside the field's Arc, is told to build it in a factory
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "field 'archive' has a trait-object type, which has no constructor; add build = factory to the field and build it in the build_archive method of the container's factories trait"
    And the error should point at line <line> of the fixture
    And the compiler should report 1 errors

    Examples:
      | fixture                                | line |
      | trait_object_inside_arc                | 10   |
      | trait_object_inside_arc_passed_as_type | 12   |

  @v0_1_0 @regression
  Scenario Outline: Graph errors point at the declaration that caused them
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then the error should point at line <line> of the fixture

    Examples:
      | fixture            | line |
      | unknown_dependency | 13   |
      | generic_container  | 4    |
      | dependency_cycle   | 9    |
