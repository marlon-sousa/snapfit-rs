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
      | static_dispatch_for_name_as_path         | 14   | static_dispatch_for names the dispatched type through the path shop::Db; write a single name, such as static_dispatch_for(Database as Db)          |
      | static_dispatch_for_unknown_setting      | 14   | static_dispatch_for takes the trait and its name and nothing else, not extra; remove extra, and add implementations with implementation on a field or with extend_static_dispatch, such as extend_static_dispatch(Db, [Sqlite]) |
      | static_dispatch_for_without_parentheses  | 14   | static_dispatch_for cannot be read: it needs its settings in parentheses; write it as static_dispatch_for(Database as Db)                          |
      | static_dispatch_for_after_equals         | 14   | static_dispatch_for cannot be read: it takes its settings in parentheses, not after '='; write it as static_dispatch_for(Database as Db)        |
      | static_dispatch_for_after_the_name       | 14   | static_dispatch_for cannot be read: it found Extra after the name, where only a closing comma may follow; write it as static_dispatch_for(Database as Db) |
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
      | static_dispatch_without_implementations | 15   | the dispatched type 'Unused' holds no implementation in this build; name one with implementation on a field of type Arc<Unused>, list those a factory returns with implementations = [...] beside build = factory, or add one with extend_static_dispatch(Unused, [...]) |

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
      | dispatched_list_without_factory         | 22   | field 'replica' lists implementations of the dispatched type Db, which only a factory can choose between; add build = factory to the field and return one of them from the build_replica method of the container's factories trait, or name the one it builds with implementation, such as implementation = MySql |
      | implementations_on_a_trait_object       | 19   | field 'replica' names implementations, which only a field of a dispatched type built by build = factory takes; name the one type the container builds with implementation, such as implementation = StripeGateway, or remove implementations |
      | implementations_empty_list              | 19   | field 'replica': implementations lists no type in its square brackets; list the types the field's factory may return, such as implementations = [Postgres, Sqlite], or remove implementations |
      | implementations_entry_not_a_type        | 19   | field 'replica': 'impl Database' is not a type the container can build; write each type in full, such as implementations = [Postgres, Sqlite] |
      | implementations_entry_quoted            | 19   | field 'replica': implementations takes types, not strings; write each unquoted, such as implementations = [Postgres] |
      | implementations_entry_empty             | 19   | field 'replica': implementations has an entry with no type between two commas; remove the extra comma, or write the type |
      | dispatched_implementation_with_factory  | 19   | field 'replica' names the one implementation MySql of the dispatched type Db, and is built with build = factory, which returns the field itself; list the variants its factory may return with implementations = [MySql], or remove build = factory so that the container builds MySql |
      | implementation_given_a_list             | 19   | field 'replica': implementation takes one type, not a list; to list the types a factory may return, write implementations, with an s, such as implementations = [Postgres, Sqlite] |
      | implementations_without_a_list          | 19   | field 'replica': implementations takes its types in square brackets, such as implementations = [Postgres, Sqlite]; to name the one type the container builds, write implementation, without an s, such as implementation = Postgres |
      | implementation_beside_implementations   | 19   | field 'replica': implementation and implementations are both given; keep implementation for the one type the container builds, or implementations, beside build = factory, for the types its factory may return |
      | implementations_repeated                | 19   | field 'replica': implementations is given more than once; list every type in one implementations setting |
      | dispatched_field_through_path           | 19   | field 'replica' names the dispatched type Db through the path crate::Db; write Db as static_dispatch_for declares it                                                                                                                                |
      | static_dispatch_for_written_by_a_macro  | 22   | field 'replica' holds the dispatched type Db, whose implementations the container cannot choose between                                                                                                                                              |

  @v0_3_0
  Scenario Outline: A refused container still declares its dispatched types, so that its fields and the code that uses them raise no error of their own
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<message>"
    And the error should point at line <line> of the fixture
    And the compiler should report <count> errors

    Examples:
      | fixture                                      | line | count | message                                                                                    |
      | static_dispatch_for_setting_beside_its_fields | 14 | 1    | static_dispatch_for takes the trait and its name and nothing else, not extra |
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
  Scenario Outline: A trait without #[snapfit::static_dispatch], or whose implementations disagree on an associated type, is refused in words at the trait the declaration names
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "`Db` cannot dispatch the trait `Database` that static_dispatch_for names, because the trait, or one of its supertraits, is not marked #[snapfit::static_dispatch], an implementation on its list does not implement it, or the implementations disagree on an associated type"
    And the error should point at line <line> of the fixture
    And the compiler should report 1 errors

    Examples:
      | fixture                                      | line |
      | static_dispatch_unmarked_trait               | 13   |
      | static_dispatch_disagreeing_associated_types | 20   |

  @v0_3_0 @regression
  Scenario: A trait without #[snapfit::static_dispatch] beside a marked one that shares an implementation is refused in words alone, with no note of a trait of the same name
    Given an invalid container fixture "static_dispatch_unmarked_beside_a_marked_trait"
    When the fixture is compiled
    Then compilation should fail with "`Shelf` cannot dispatch the trait `Store` that static_dispatch_for names"
    And the error should point at line 20 of the fixture
    And the compiler output should not mention "similarly named"
    And the compiler output should not mention "same name"
    And the compiler should report 1 errors

  @v0_3_0
  Scenario Outline: An implementation of another trait is rustc's own error where its list names it
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "the trait bound `Csv: Database` is not satisfied"
    And the error should point at line <line> of the fixture
    And the compiler should report <count> errors

    Examples:
      | fixture                                                | line | count |
      | static_dispatch_implementation_without_the_trait       | 18   | 2     |
      | static_dispatch_listed_implementation_without_the_trait | 19  | 2     |

  @v0_3_0
  Scenario Outline: An implementation written once with self:: and once without, which are two types inside a function, is refused at its second spelling
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<message>"
    And the error should point at line 20 of the fixture
    And the compiler should report 1 errors

    Examples:
      | fixture                                                | message                                                                                                                                                   |
      | static_dispatch_respelled_as_another_type              | `main::Archive` and `Archive` are written as one implementation of a dispatched type, since they differ only by `self::`, but they are two different types here |
      | static_dispatch_respelled_without_self_as_another_type | `Archive` and `main::Archive` are written as one implementation of a dispatched type, since they differ only by `self::`, but they are two different types here |

  @v0_3_0 @regression
  Scenario: One implementation written two ways, which of cannot choose between, is rustc's own error at the field it builds
    Given an invalid container fixture "static_dispatch_one_implementation_spelled_twice"
    When the fixture is compiled
    Then compilation should fail with "type annotations needed"
    And the error should point at line 17 of the fixture
    And the compiler should report 1 errors

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
