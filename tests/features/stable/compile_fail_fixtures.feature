Feature: Compile-fail fixtures are checked against the snapfit under test

  As a contributor running snapfit's test suite
  I want each compile-fail fixture compiled against the build of snapfit the suite runs
  So that another build of snapfit left in the target folder never changes what a scenario sees

  @regression
  Scenario: A fixture keeps its refusal beside a newer snapfit library from another build
    Given a newer snapfit library from another build sits beside the one the suite links to
    And an invalid container fixture "dependency_cycle"
    When the fixture is compiled
    Then compilation should fail with "dependency cycle: service_a depends on service_b, which depends on service_a"
    And the error should point at line 9 of the fixture
