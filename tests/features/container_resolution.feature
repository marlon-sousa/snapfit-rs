Feature: Container Dependency Resolution

  As a Rust developer building services with wireup
  I want container-managed singletons to resolve topologically
  So that dependent components receive shared instances without domain pollution

  @mvp
  Scenario: Singletons are instantiated once and shared by reference
    Given an AppContainer with leaf Config, Database, and UserService
    When the container is instantiated
    Then the container should provide a Config singleton
    And the container should provide a Database singleton
    And the container should provide a UserService singleton
    And the Database singleton should reference the exact same Config instance as the container
    And the UserService singleton should reference the exact same Database instance as the container

  @mvp
  Scenario: Multiple dependencies of the same type are distinguished by field name
    Given an AppContainer with "read_db" and "write_db" singletons of type Database
    When the container is instantiated
    Then the container should provide separate "read_db" and "write_db" instances
    And the "read_db" instance should have a different memory address from the "write_db" instance
