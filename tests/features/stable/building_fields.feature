Feature: Choosing how a field is built

  As a Rust developer whose dependency is not built by its type's new
  I want one build setting that says how the container builds the field, down to a factory I write myself
  So that the container calls the right code, with each dependency of a factory passed by name, and the compiler checks the names

  @v0_1_0
  Scenario: A field is built by Default, or by a constructor with another name
    When the books whose tax rates are built by Default and whose ledger is opened by its open constructor are instantiated
    Then the ledger should be opened by open with the container's own default tax rates

  @v0_1_0
  Scenario: A factory receives two dependencies of the same type by name, whatever order deps lists them in
    When the shop whose order store is built by a factory from write_db and read_db is instantiated
    Then the order store should read from the container's read_db and write to its write_db
    And the order store should be built once and shared

  @v0_1_0
  Scenario: A factory builds a trait-object field that other fields depend on
    When the store whose trait-object payments field is built by a factory is instantiated
    Then the checkout should charge through the factory's gateway, reading "northwind charged 250 cents in the sandbox"

  @v0_1_0
  Scenario: A container builds every field without deps or a build of its own as its leaf_build says
    When the billing container whose leaves are built by Default is instantiated
    Then its leaves should be built by Default, its mailer by its own constructor, and its invoicer by new

  @v0_1_0
  Scenario: A container's leaf_build can name a constructor for every leaf
    When the archive container whose leaf_build is constructor(open) is instantiated
    Then its leaves should be opened by open, and its leaf that says build = default should not be

  @v0_1_0
  Scenario: A bad leaf_build is reported together with the container's other bad settings
    Given an invalid container fixture "several_container_settings"
    When the fixture is compiled
    Then compilation should fail with "invalid container mode 'threaded': write mode = "sync", or leave mode out"
    And compilation should fail with "leaf_build = factory cannot build the container's fields that have no deps"
    And the compiler should report 2 errors

  @v0_1_0
  Scenario Outline: A build setting that cannot be followed is refused at its field
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error>"
    And the error should point at line <line> of the fixture

    Examples:
      | fixture            | line | expected_error                                                                                                                                                  |
      | default_with_deps  | 15   | field 'rates' is built with build = default, which takes no dependencies; remove its deps, or build it another way                                              |
      | invalid_leaf_build | 7    | leaf_build = factory cannot build the container's fields that have no deps; write leaf_build = default, or leaf_build = constructor with the function's name in brackets, such as leaf_build = constructor(open) |
      | unknown_build      | 8    | build = factroy on field 'db' is not a way to build a field; write build = default, build = factory, or build = constructor(factroy) to call a function of that name |
      | quoted_build       | 9    | build "default" on field 'rates' must be written without quotes: build = default |
      | constructor_without_name | 8 | build = constructor on field 'db' needs the function's name, such as build = constructor(connect) |
      | not_a_build        | 8    | build = Database :: connect on field 'db' is not a way to build a field; write build = default, build = factory, or build = constructor(connect) |

  @v0_1_0
  Scenario: A misspelt input of a factory is a compile error naming the inputs struct
    Given an invalid container fixture "factory_input_misnamed"
    When the fixture is compiled
    Then compilation should fail with "struct `ShopOrdersInputs` does not have a field named `reed_db`"
    And the error should point at line 25 of the fixture

  @v0_1_0
  Scenario: A factory field whose trait is not implemented is a compile error naming the trait
    Given an invalid container fixture "factory_not_implemented"
    When the fixture is compiled
    Then compilation should fail with "the trait bound `Shop: ShopFactories` is not satisfied"
    And the error should point at line 9 of the fixture

  @v0_1_0
  Scenario Outline: The settings that build replaced are gone
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error>"
    And the error should point at line 8 of the fixture

    Examples:
      | fixture                     | expected_error            |
      | removed_builder             | Unknown field: `builder`  |
      | removed_constructor_setting | Unknown field: `constructor` |
