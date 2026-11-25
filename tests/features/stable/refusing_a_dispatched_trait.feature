Feature: Refusing a trait that static dispatch cannot follow

  As a Rust developer marking a trait for static dispatch
  I want each item that a match cannot forward refused at that item, in words, beside every other problem
  So that I fix the whole trait in one pass instead of reading errors in code snapfit wrote

  @v0_3_0
  Scenario Outline: An item that a match cannot forward is refused at the item
    Given an invalid dispatched trait fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<message>"
    And the error should point at line <line> of the fixture
    And the compiler should report 1 errors

    Examples:
      | fixture                                  | line | message                                                                                                                                                                                                   |
      | static_dispatch_no_receiver              | 5    | method 'create' in trait 'Database' has no receiver, so a dispatched value has no implementation to call it on; take self, &self or &mut self, or move the function out of the trait              |
      | static_dispatch_associated_const         | 3    | associated const 'LIMIT' in trait 'Database' could have a different value in each implementation, so static dispatch cannot forward it; make it a method instead, such as fn limit(&self) -> usize |
      | static_dispatch_generic_associated_type  | 6    | associated type 'Rows' in trait 'Database' has generic parameters, which static dispatch does not forward in this version of snapfit; remove its parameters                                         |
      | static_dispatch_impl_trait_return        | 5    | method 'orders' in trait 'Database' returns impl Trait, and each implementation would return a type of its own; return a named type instead, such as Box<dyn Iterator<Item = Order>>              |
      | static_dispatch_macro_item               | 11   | the call to the macro 'sources!' in trait 'Database' writes items that snapfit cannot see, so it cannot forward them; write the items in the trait itself                                          |

  @v0_3_0
  Scenario: A receiver other than self, &self or &mut self is refused at each method
    Given an invalid dispatched trait fixture "static_dispatch_boxed_receiver"
    When the fixture is compiled
    Then compilation should fail with "method 'close' in trait 'Database' takes self through a type that static dispatch cannot forward, such as Box<Self>; take self, &self or &mut self instead"
    And compilation should fail with "method 'poll' in trait 'Database' takes self through a type that static dispatch cannot forward, such as Box<Self>; take self, &self or &mut self instead"
    And the error should point at line 5 of the fixture
    And the error should point at line 7 of the fixture
    And the compiler should report 2 errors

  @v0_3_0
  Scenario: Self in an argument or inside a return type is refused, and a Self:: path is not
    Given an invalid dispatched trait fixture "static_dispatch_self_elsewhere"
    When the fixture is compiled
    Then compilation should fail with "method 'merge' in trait 'Database' uses Self in its arguments or inside its return type, where two dispatched values could hold different implementations; use Self only as the receiver or as the whole return type"
    And compilation should fail with "method 'replica' in trait 'Database' uses Self in its arguments or inside its return type"
    And the error should point at line 3 of the fixture
    And the error should point at line 7 of the fixture
    And the compiler should report 2 errors

  @v0_3_0
  Scenario: An unsafe trait is refused at its unsafe
    Given an invalid dispatched trait fixture "static_dispatch_unsafe_trait"
    When the fixture is compiled
    Then compilation should fail with "trait 'Database' is unsafe, and snapfit cannot promise that its contract holds for the type that dispatches it; remove unsafe from the trait, or dispatch it dynamically"
    And the error should point at line 2 of the fixture
    And the compiler should report 1 errors

  @v0_3_0
  Scenario: The attribute on anything but a trait is refused at the item
    Given an invalid dispatched trait fixture "static_dispatch_on_struct"
    When the fixture is compiled
    Then compilation should fail with "#[snapfit::static_dispatch] goes on a trait, and this item is not one; put it on the trait the container dispatches"
    And the error should point at line 2 of the fixture
    And the compiler should report 1 errors

  @v0_3_0
  Scenario: An argument is refused, and remote is refused as planned
    Given an invalid dispatched trait fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<message>"
    And the error should point at line 1 of the fixture
    And the compiler should report 1 errors

    Examples:
      | fixture                       | message                                                                                                    |
      | static_dispatch_with_argument | #[snapfit::static_dispatch] takes no arguments; write it alone, as #[snapfit::static_dispatch]             |
      | static_dispatch_remote        | #[snapfit::static_dispatch(remote = ...)] is planned for a future version of snapfit and is not implemented yet |

  @v0_3_0
  Scenario: Every problem of the attribute and of the trait is reported in one compile
    Given an invalid dispatched trait fixture "static_dispatch_several_problems"
    When the fixture is compiled
    Then compilation should fail with "#[snapfit::static_dispatch] takes no arguments"
    And compilation should fail with "associated const 'LIMIT' in trait 'Database' could have a different value in each implementation"
    And compilation should fail with "method 'create' in trait 'Database' has no receiver"
    And compilation should fail with "method 'orders' in trait 'Database' returns impl Trait"
    And the error should point at line 1 of the fixture
    And the error should point at line 3 of the fixture
    And the error should point at line 5 of the fixture
    And the error should point at line 9 of the fixture
    And the compiler should report 4 errors

  @v0_3_0
  Scenario: A trait that a macro_rules! declares is refused at the item the macro writes, and Self passed as a type is its whole return type
    Given an invalid dispatched trait fixture "static_dispatch_written_by_a_macro"
    When the fixture is compiled
    Then compilation should fail with "method 'create' in trait 'Database' has no receiver"
    And the error should point at line 7 of the fixture
    And the compiler should report 1 errors

  @v0_3_0
  Scenario: Each call to the same macro among the items is refused at its own line
    Given an invalid dispatched trait fixture "static_dispatch_repeated_macro"
    When the fixture is compiled
    Then compilation should fail with "the call to the macro 'sources!' in trait 'Database' writes items that snapfit cannot see"
    And the error should point at line 9 of the fixture
    And the error should point at line 13 of the fixture
    And the compiler should report 2 errors

  @v0_3_0
  Scenario: A method bounding Self by a trait other than Sized is refused at the method
    Given an invalid dispatched trait fixture "static_dispatch_self_bound"
    When the fixture is compiled
    Then compilation should fail with "method 'copied' in trait 'Database' bounds Self by a trait other than Sized, which snapfit cannot promise of every implementation the dispatched type holds; remove the bound, or make it a supertrait of the trait"
    And the error should point at line 5 of the fixture
    And the compiler should report 1 errors

  @v0_3_0
  Scenario: An associated type naming Self in its bounds, bounding Self by a trait other than Sized, or declared under cfg, is refused at the type
    Given an invalid dispatched trait fixture "static_dispatch_associated_type_bounds"
    When the fixture is compiled
    Then compilation should fail with "associated type 'Twin' in trait 'Database' names Self in its bounds, which snapfit cannot promise of the type that dispatches the trait; remove Self from its bounds"
    And compilation should fail with "associated type 'Row' in trait 'Database' is declared under #[cfg] or #[cfg_attr], which the bound that makes every implementation agree on it cannot follow; declare it in every build"
    And compilation should fail with "associated type 'Pair' in trait 'Database' bounds Self by a trait other than Sized"
    And the error should point at line 3 of the fixture
    And the error should point at line 5 of the fixture
    And the error should point at line 8 of the fixture
    And the compiler should report 3 errors
