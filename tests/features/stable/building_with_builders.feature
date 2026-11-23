Feature: Building a field with its type's builder

  As a Rust developer whose type is built through a builder, from a crate such as bon or typed-builder or written by hand
  I want the container to pass each dependency to the builder's setter of that name
  So that two dependencies of the same type can never take each other's place, and the compiler checks every name

  @v0_1_0 @regression
  Scenario: Labels pass two parts of the same trait to their own setters, whatever order deps lists them in
    When the site whose writer's builder takes memory_files as memory and folder_files as disk is instantiated
    Then the site writer should write to disk through folder_files and to memory through memory_files

  @v0_1_0
  Scenario: Each dependency without a label is passed to the setter of its own name
    When the shelves whose catalog's builder takes write_db and read_db by their own names are instantiated
    Then the catalog should read from the container's read_db and write to its write_db

  @v0_1_0
  Scenario: A hand-written builder names the functions its chain starts and finishes with
    When the reports whose report's hand-written builder starts with draft and finishes with publish are instantiated
    Then the report should be published with the container's config

  @v0_1_0
  Scenario Outline: A builder setting that cannot be followed is refused at its field
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error>"
    And the error should point at line <line> of the fixture

    Examples:
      | fixture                 | line | expected_error |
      | label_without_builder   | 20   | the dependency 'disk = folder_files' of field 'site_reader' has a label, which names a builder setter; write folder_files alone, or build the field with build = builder |
      | duplicate_setter        | 20   | field 'site_writer' passes more than one dependency to the builder setter 'disk'; give each dependency its own setter |
      | invalid_builder_setting | 8    | builder(begin = make, finish = "done") on field 'exporter' cannot be read; builder takes start and finish, each with a function's name, such as builder(start = new_builder, finish = done) |
      | builder_trait_object    | 8    | field 'writer' has a trait-object type, which has no constructor; name the type the container builds, such as implementation = StripeGateway, or add build = factory to the field and build it in the build_writer method of the container's factories trait |

  @v0_1_0
  Scenario: A setter the builder does not have is the compiler's error at its label
    Given an invalid container fixture "builder_unknown_setter"
    When the fixture is compiled
    Then compilation should fail with "no method named `disc` found"
    And the error should point at line 22 of the fixture

  @v0_1_0
  Scenario: A setter a typestate builder needs and deps leaves out is a compile error
    Given an invalid container fixture "builder_missing_setter"
    When the fixture is compiled
    Then compilation should fail with "was not set"
    And the error should point at line 22 of the fixture
