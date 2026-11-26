Feature: Reaching snapfit under another name

  As a Rust developer whose crate depends on snapfit under another name, or reaches it through a crate of its own
  I want to tell #[container] and #[snapfit::static_dispatch] the path to snapfit
  So that the code they write compiles without a dependency named snapfit

  @v0_3_0
  Scenario Outline: A crate that depends on snapfit only under the name di builds its containers and dispatched types
    Given a fixture "<fixture>" of a crate that depends on snapfit only under the name di
    When that crate is linted with clippy
    Then clippy should report no warning

    Examples:
      | fixture            |
      | renamed_dependency |

  @v0_3_0
  Scenario: A container refused in such a crate still writes its dispatched type through di
    Given a fixture "renamed_dependency_refused" of a crate that depends on snapfit only under the name di
    When that crate is linted with clippy
    Then compilation should fail with "invalid container mode 'threaded': write mode = "sync", or leave mode out"
    And the error should point at line 21 of the fixture
    And the compiler should report 1 errors

  @v0_3_0
  Scenario: A quoted crate is refused with the path written unquoted
    Given an invalid container fixture "crate_setting_quoted"
    When the fixture is compiled
    Then compilation should fail with "crate takes the path snapfit is reached through, not a string; write it unquoted: crate = snapfit"
    And the error should point at line 7 of the fixture
    And the compiler should report 1 errors

  @v0_3_0
  Scenario: A crate that is no path is refused together with the container's other bad settings
    Given an invalid container fixture "crate_setting_not_a_path"
    When the fixture is compiled
    Then compilation should fail with "crate takes the path snapfit is reached through, and 'snapfit::<u32>()' is not a path; write the name the crate depends on snapfit under, such as crate = di"
    And compilation should fail with "invalid container mode 'threaded': write mode = "sync", or leave mode out"
    And the error should point at line 7 of the fixture
    And the compiler should report 2 errors
