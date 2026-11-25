Feature: Dispatching a container's fields statically

  As a Rust developer whose services take one type for a closed list of implementations
  I want a container to declare that type, and each field typed with it to name the implementation it holds
  So that every call reaches its implementation through a match, and swapping one is a single word

  @v0_3_0
  Scenario Outline: Each dispatched field holds the implementation it names, built as that implementation is
    Given a container whose fields hold a statically dispatched ledger
    Then its <field> ledger should answer from "<source>"

    Examples:
      | field  | source                   |
      | main   | disk                     |
      | backup | remote at ledger.example |
      | cache  | memory                   |

  @v0_3_0
  Scenario: A dependent receives the dispatched type and calls the trait on it
    Given a container whose fields hold a statically dispatched ledger
    Then its books should read from "disk"

  @v0_3_0
  Scenario: A field typed as the trait object stays dynamic beside the dispatched ones
    Given a container whose fields hold a statically dispatched ledger
    Then its dynamic ledger should answer from "disk"

  @v0_3_0
  Scenario Outline: A test builds an implementation no field injects, from extra or from an extension whose condition holds
    Given a container whose fields hold a statically dispatched ledger
    Then a ledger built from <list> should answer from "<source>"

    Examples:
      | list          | source |
      | extra         | fake   |
      | the extension | spare  |

  @v0_3_0
  Scenario: A fallible dispatched field returns its error from try_new
    Then building the checked accounts should fail with "snapfit could not build the field `remote` of `CheckedAccounts`"

  @v0_3_0
  Scenario: A container that a macro_rules! writes dispatches the types the macro passes
    Then a container a macro_rules! writes should answer from "disk", and its extra from "fake"

  @v0_3_0
  Scenario: Neither rustc nor clippy warns about the code snapfit writes for a dispatched type, however private its implementations are
    Given a container fixture "static_dispatch_container" for clippy
    When the fixture is linted with clippy
    Then clippy should report no warning
