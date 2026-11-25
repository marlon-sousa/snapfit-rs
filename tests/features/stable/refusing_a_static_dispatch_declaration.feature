Feature: Refusing a static dispatch declaration that cannot be followed

  As a Rust developer declaring the types a container dispatches statically
  I want each declaration, extension or field that snapfit cannot follow refused where it is written, in words, beside every other problem
  So that I fix the whole container in one pass instead of reading errors in code snapfit wrote

  @v0_3_0
  Scenario Outline: A dispatch attribute that cannot be read is refused at the attribute
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<message>"
    And the error should point at line <line> of the fixture
    And the compiler should report 1 errors

    Examples:
      | fixture                                  | line | message                                                                                                                                                              |
      | static_dispatch_for_without_name         | 14   | static_dispatch_for needs the name the fields write, after the trait and the word as, such as static_dispatch_for(Database as Db)                                    |
      | static_dispatch_for_name_as_path         | 14   | static_dispatch_for names the dispatched type through the path shop::Db; write a single name, such as static_dispatch_for(Database as Db, extra = [Sqlite])          |
      | static_dispatch_for_unknown_setting      | 14   | static_dispatch_for takes extra and nothing else after the trait and its name, not extras; write extra = [Sqlite] to add implementations, or remove extras            |
      | static_dispatch_for_repeated_extra       | 14   | static_dispatch_for gives extra more than once; list every extra implementation in one extra = [...]                                                                 |
      | static_dispatch_for_extra_not_a_list     | 14   | static_dispatch_for takes its implementations in square brackets, not MySql; write them as static_dispatch_for(Database as Db, extra = [Sqlite])                     |
      | static_dispatch_for_extra_not_a_type     | 14   | impl Database in static_dispatch_for is not a type the container can name; write each implementation in full, such as static_dispatch_for(Database as Db, extra = [Sqlite]) |
      | static_dispatch_for_without_parentheses  | 14   | static_dispatch_for cannot be read: it needs its settings in parentheses; write it as static_dispatch_for(Database as Db, extra = [Sqlite])                          |
      | static_dispatch_for_after_equals         | 14   | static_dispatch_for cannot be read: it takes its settings in parentheses, not after '='; write it as static_dispatch_for(Database as Db, extra = [Sqlite])        |
      | static_dispatch_for_after_the_name       | 14   | static_dispatch_for cannot be read: it found Extra after the name, where only a comma and the settings may follow; write it as static_dispatch_for(Database as Db, extra = [Sqlite]) |
      | extend_static_dispatch_without_list      | 15   | extend_static_dispatch(Db) needs the implementations it adds, in square brackets after the name, such as extend_static_dispatch(Db, [Mock])                          |
      | extend_static_dispatch_name_as_path      | 15   | extend_static_dispatch names the dispatched type through the path shop::Db; write a single name, such as extend_static_dispatch(Db, [Mock])                         |
      | extend_static_dispatch_not_a_list        | 15   | extend_static_dispatch takes its implementations in square brackets, not MySql; write them as extend_static_dispatch(Db, [Mock])                                    |
      | extend_static_dispatch_not_a_type        | 15   | impl Database in extend_static_dispatch is not a type the container can name; write each implementation in full, such as extend_static_dispatch(Db, [Mock])       |
      | extend_static_dispatch_two_lists         | 15   | extend_static_dispatch cannot be read: it takes one list after the name, and found [MySql] after it; write it as extend_static_dispatch(Db, [Mock])                 |
      | extend_static_dispatch_without_parentheses | 15 | extend_static_dispatch cannot be read: it needs its settings in parentheses; write it as extend_static_dispatch(Db, [Mock])                                        |

  @v0_3_0
  Scenario Outline: A declaration the container cannot follow is refused at the attribute
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<message>"
    And the error should point at line <line> of the fixture
    And the compiler should report 1 errors

    Examples:
      | fixture                                 | line | message                                                                                                                                        |
      | static_dispatch_repeated_name           | 15   | the dispatched type 'Db' is declared more than once; give each static_dispatch_for a name of its own                                           |
      | extend_static_dispatch_undeclared       | 15   | extend_static_dispatch extends 'Dbs', which this container does not declare; extend one of the types its static_dispatch_for declares: [Db]    |
      | extend_static_dispatch_with_nothing_declared | 14 | extend_static_dispatch extends 'Db', which this container does not declare; declare it first, such as static_dispatch_for(Database as Db) |
      | static_dispatch_without_implementations | 15   | the dispatched type 'Unused' holds no implementation in this build; name one with implementation on a field of type Arc<Unused>, or list it in extra = [...] |

  @v0_3_0
  Scenario Outline: A field of a dispatched type that the container cannot build is refused at the field
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<message>"
    And the error should point at line <line> of the fixture
    And the compiler should report 1 errors

    Examples:
      | fixture                                 | line | message                                                                                                                                                                                                                                              |
      | dispatched_field_without_implementation | 19   | field 'replica' holds the dispatched type Db, whose implementations the container cannot choose between; name the one it builds, such as implementation = MySql, or add build = factory to the field and build it in the build_replica method of the container's factories trait |
      | dispatched_implementation_with_factory  | 19   | field 'replica' names an implementation and is built with build = factory, which returns the field itself; remove implementation, or remove build = factory so that the container builds the implementation                                                                   |
      | dispatched_field_through_path           | 19   | field 'replica' names the dispatched type Db through the path crate::Db; write Db as static_dispatch_for declares it                                                                                                                                |
      | static_dispatch_for_written_by_a_macro  | 21   | field 'replica' holds the dispatched type Db, whose implementations the container cannot choose between                                                                                                                                              |

  @v0_3_0
  Scenario Outline: A refused container still declares its dispatched types, so that its fields and the code that uses them raise no error of their own
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<message>"
    And the error should point at line <line> of the fixture
    And the compiler should report <count> errors

    Examples:
      | fixture                                      | line | count | message                                                                                    |
      | static_dispatch_for_unreadable_extra_beside_its_fields | 14 | 1 | static_dispatch_for takes its implementations in square brackets, not MySql |
      | dispatched_field_beside_an_unannotated_field | 19   | 1     | field 'plain' has no snapfit attribute; add #[singleton] to have the container build it    |
      | dispatched_type_held_by_no_field             | 14   | 2     | the dispatched type 'Db' holds no implementation in this build                             |
      | dispatched_type_held_by_no_field             | 16   | 2     | field 'replica' holds the dispatched type Db, whose implementations the container cannot choose between |

  @v0_3_0
  Scenario: Every problem of the dispatch declarations and their fields is reported at once
    Given an invalid container fixture "static_dispatch_several_container_problems"
    When the fixture is compiled
    Then compilation should fail with "the dispatched type 'Db' is declared more than once"
    And compilation should fail with "extend_static_dispatch extends 'Dbs', which this container does not declare"
    And compilation should fail with "field 'replica' holds the dispatched type Db"
    And the error should point at line 15 of the fixture
    And the error should point at line 16 of the fixture
    And the error should point at line 21 of the fixture
    And the compiler should report 3 errors

  @v0_3_0
  Scenario: A trait without #[snapfit::static_dispatch] is rustc's own error at the trait the declaration names
    Given an invalid container fixture "static_dispatch_unmarked_trait"
    When the fixture is compiled
    Then compilation should fail with "Only<MySql>: Database` is not satisfied"
    And the error should point at line 13 of the fixture
    And the compiler should report 1 errors

  @v0_3_0
  Scenario: Implementations that disagree on an associated type are rustc's own error at the trait the declaration names
    Given an invalid container fixture "static_dispatch_disagreeing_associated_types"
    When the fixture is compiled
    Then compilation should fail with "as Database>::Row == u32`"
    And the error should point at line 20 of the fixture
    And the compiler should report 1 errors

  @v0_3_0
  Scenario: An implementation of another trait is rustc's own error where its list names it
    Given an invalid container fixture "static_dispatch_implementation_without_the_trait"
    When the fixture is compiled
    Then compilation should fail with "the trait bound `Csv: Database` is not satisfied"
    And the error should point at line 17 of the fixture
    And the compiler should report 2 errors

  @v0_3_0
  Scenario: A dispatch attribute written above #[container] is an attribute rustc cannot find, which snapfit reads as well
    Given an invalid container fixture "static_dispatch_for_above_container"
    When the fixture is compiled
    Then compilation should fail with "cannot find attribute `static_dispatch_for` in this scope"
    And compilation should fail with "the dispatched type 'Db' holds no implementation in this build"
    And the error should point at line 13 of the fixture
    And the compiler should report 2 errors

  @v0_3_0
  Scenario: The #[dispatch] attribute planned before 0.3.0 is refused, naming static_dispatch_for
    Given an invalid container fixture "dispatch_attribute"
    When the fixture is compiled
    Then compilation should fail with "field 'store' is marked #[dispatch(...)], which snapfit does not support; declare the dispatched type under #[container] with static_dispatch_for, such as #[static_dispatch_for(Store as Stores)], and give the field that type with its implementation, such as Arc<Stores>"
    And the error should point at line 10 of the fixture
    And the compiler should report 1 errors
