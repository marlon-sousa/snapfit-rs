Feature: Building a container whose fields can fail

  As a Rust developer whose services read files, open connections or check their settings when they start
  I want to mark the fields whose constructor, builder or factory returns a Result as fallible
  So that the container hands me the first failure, naming the field it came from, instead of panicking

  @v0_2_0
  Scenario: A container whose fallible fields all succeed is built by try_new
    When the shop whose configuration, tax rates, database and payment gateway can each fail is opened and none fails
    Then every field should be built from the value its constructor, builder or factory returned
    And its receipt printer should be built by Default, and its tax rates, a fallible leaf with a build of its own, by their own new

  @v0_2_0
  Scenario: A leaf_build written constructor(new) builds a fallible leaf by its own new, as leaving leaf_build out would
    When the till whose leaf_build is written constructor(new) and whose ledger can fail is opened
    Then its ledger should be built by the ledger's own new

  @v0_2_0 @regression
  Scenario Outline: A fallible field of a container that a macro_rules! writes from tokens is built
    When the tills that a macro_rules! writes from the tokens it is passed are opened
    Then the till passed "<form>" should have built its ledger by the ledger's own new

    Examples:
      | form              |
      | the whole struct  |
      | the fallible word |

  @v0_2_0
  Scenario: The first field that fails stops the build and is named in the error
    When the storefront whose database fails to connect after its configuration is built is opened
    Then the error should say that snapfit could not build the field db of Storefront
    And its source should be the database's own error, saying that port 5432 is taken
    And the configuration should have been built and dropped, and the order service never built

  @v0_2_0
  Scenario Outline: A fallible field written in a way snapfit cannot follow is refused at its field
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error>"
    And the error should point at line <line> of the fixture
    And the compiler should report <errors> errors

    Examples:
      | fixture                     | line | errors | expected_error |
      | bare_fallible               | 9    | 1      | field 'config': fallible needs a value; write fallible = true to have the container return the field's error, or leave fallible out |
      | not_a_fallible_value        | 9    | 1      | field 'config': fallible takes true or false, not "yes"; write fallible = true, or fallible = false |
      | fallible_default            | 9    | 1      | field 'rates' is built with build = default, which cannot fail; remove fallible = true, or build it another way |
      | fallible_leaf_build         | 14   | 1      | field 'config' is built with the container's leaf_build, since it has no deps and no build of its own, and leaf_build cannot fail; give the field a build of its own, such as build = constructor(new), or remove fallible = true |
      | planned_fallible_transient  | 16   | 2      | fallible = true on transient dependency 'request' is planned for a future version of snapfit and is not implemented yet |

  @v0_2_0
  Scenario: A fallible field whose constructor returns a plain value is a compile error at its fallible setting
    Given an invalid container fixture "fallible_plain_value"
    When the fixture is compiled
    Then compilation should fail with "the field is marked `fallible = true`, so what builds it must return a `Result`, not `Config`"
    And the error should point at line 14 of the fixture
    And the compiler should report 1 errors

  @v0_2_0
  Scenario: A container with a fallible field has no new, which could not report its error
    Given an invalid container fixture "fallible_container_new"
    When the fixture is compiled
    Then compilation should fail with "named `new` found for struct `Shop`"
    And the error should point at line 19 of the fixture
    And the compiler should report 1 errors

  @v0_2_0
  Scenario: A fallible field whose error does not convert into a boxed error is a compile error at its fallible setting
    Given an invalid container fixture "fallible_error_not_an_error"
    When the fixture is compiled
    Then compilation should fail with "the trait bound `(): std::error::Error` is not satisfied"
    And the error should point at line 14 of the fixture
    And the compiler should report 1 errors

  @v0_2_0
  Scenario: A fallible field whose Result holds another type than the field is a type mismatch, without a question mark the user never wrote
    Given an invalid container fixture "fallible_wrong_value"
    When the fixture is compiled
    Then compilation should fail with "mismatched types"
    And the error should point at line 15 of the fixture
    And the compiler should report 1 errors
    And the compiler output should not mention "`?` operator"
