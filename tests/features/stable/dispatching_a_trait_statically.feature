Feature: Dispatching a trait statically

  As a Rust developer whose services call one trait through a closed list of its implementations
  I want the trait marked once, where it is defined, to be implemented for snapfit's dispatch types
  So that a call reaches the implementation a value holds through a match, with no trait object

  @v0_3_0
  Scenario Outline: A dispatched value calls the implementation it holds
    Given a database dispatched statically, holding <database>
    Then finding order 7 should answer from <database>
    And its name should be <database>
    And tagging it with 3 should read "<database>:3"
    And showing "a receipt" through it should read "<database> shows a receipt"
    And pinging it should read "<database> answers"

    Examples:
      | database |
      | mysql    |
      | postgres |
      | mock     |

  @v0_3_0
  Scenario Outline: A default method is called on the implementation, which may override it
    Given a database dispatched statically, holding <database>
    Then its greeting should read "<greeting>"

    Examples:
      | database | greeting         |
      | mysql    | hello from mysql |
      | mock     | the mock greets  |

  @v0_3_0
  Scenario: A method that takes &mut self changes the implementation the value holds
    Given a database dispatched statically, holding postgres
    When it stores the orders 1 and 2
    Then it should hold the orders 1 and 2

  @v0_3_0
  Scenario: A method that takes self and a method that returns Self keep to the variant
    Given a database dispatched statically, holding mock
    When it is renamed with the prefix "test "
    Then it should still hold the mock
    And taken by value, it should give its name "test mock"

  @v0_3_0
  Scenario Outline: A trait with a type parameter and a lifetime of its own is dispatched
    Given shelves dispatched statically, holding <store>
    When two items are put on them
    Then the last put should answer <answer>

    Examples:
      | store    | answer |
      | memory   | 2      |
      | counting | 20     |

  @v0_3_0
  Scenario: A trait that a macro_rules! declares is dispatched with the types the macro passes
    Given a rack dispatched statically, holding oak, whose trait a macro_rules! declares
    Then the rack should read from "oak"
    And its copy should still hold oak

  @v0_3_0 @regression
  Scenario: A method that a cfg_attr removes from the trait is removed from the impls snapfit writes, and one it keeps is forwarded without the other attributes the cfg_attr writes
    When a lamp whose trait a cfg_attr removes a method from is dispatched statically, holding a bulb
    Then the lamp should shine "bulb"
    And it should glow 1

  @v0_3_0 @regression
  Scenario: A cfg_attr whose condition and cfg a macro_rules! passes removes the method from the impls snapfit writes too
    When a torch whose trait a macro_rules! writes with a cfg_attr is dispatched statically, holding a bulb
    Then the lamp should shine "bulb beam"

  @v0_3_0 @regression
  Scenario: A trait whose supertrait names Self::Item and whose where clause bounds Self itself is dispatched
    Given a dispatched trait fixture "static_dispatch_self_bounds" for clippy
    When the fixture is linted with clippy
    Then clippy should report no warning

  @v0_3_0
  Scenario: Clippy does not lint the impls snapfit writes for a dispatched trait
    Given a dispatched trait fixture "static_dispatch_trait" for clippy
    When the fixture is linted with clippy
    Then clippy should report no warning

  @v0_3_0
  Scenario Outline: A value of a type that is not on the list is refused where it is wrapped, also on a list of one
    Given an invalid dispatched trait fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "`Sqlite` is not one of the implementations this dispatched type holds"
    And the error should point at line <line> of the fixture
    And the compiler should report 1 errors

    Examples:
      | fixture                              | line |
      | static_dispatch_not_on_the_list      | 27   |
      | static_dispatch_not_on_a_list_of_one | 20   |

  @v0_3_0
  Scenario Outline: A supertrait the dispatch types cannot implement is refused in words at the supertrait
    Given an invalid dispatched trait fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "snapfit cannot dispatch the supertrait `<supertrait>` of `<trait>` statically, because it is not marked #[snapfit::static_dispatch] or it is a trait of another crate"
    And the error should point at line <line> of the fixture
    And the compiler should report 1 errors

    Examples:
      | fixture                             | supertrait | trait    | line |
      | static_dispatch_unmarked_supertrait | Named      | Database | 6    |
      | static_dispatch_foreign_supertrait  | Display    | Shown    | 4    |
