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
  Scenario Outline: A trait with several type arguments is dispatched to a field held in an Arc or as a value
    Given a container whose converters a trait with two type arguments dispatches
    Then its <field> converter should turn 4 into <result>

    Examples:
      | field  | result |
      | shared | 8      |
      | owned  | 12     |

  @v0_3_0
  Scenario: A dependent receives the dispatched type and calls the trait on it
    Given a container whose fields hold a statically dispatched ledger
    Then its books should read from "disk"

  @v0_3_0
  Scenario: A field typed as the trait object stays dynamic beside the dispatched ones
    Given a container whose fields hold a statically dispatched ledger
    Then its dynamic ledger should answer from "disk"

  @v0_3_0
  Scenario Outline: A test builds an implementation no field injects, from an extension with no condition or one whose condition holds
    Given a container whose fields hold a statically dispatched ledger
    Then a ledger built from <list> extension should answer from "<source>"

    Examples:
      | list              | source |
      | the unconditional | fake   |
      | the conditional   | spare  |

  @v0_3_0
  Scenario Outline: A factory chooses the variant its field holds from a setting, among the variants its factory fields list, building it from the inputs of every variant
    Given a container whose factories choose its ledgers from the setting <backend>
    Then its shared and its owned ledger should both answer from "<source>"

    Examples:
      | backend | source                   |
      | archive | archive in ledgers       |
      | mirror  | mirror of ledger.example |
      | scratch | scratch                  |

  @v0_3_0
  Scenario: An implementation named by a field and by factories' lists is held once, and the field answers from it whatever the factories choose
    Given a container whose factories choose its ledgers from the setting mirror
    Then the ledger it builds itself should answer from "archive in ledgers"
    And its shared and its owned ledger should both answer from "mirror of ledger.example"

  @v0_3_0
  Scenario: A fallible dispatched field returns its error from try_new
    Then building the checked accounts should fail with "snapfit could not build the field `remote` of `CheckedAccounts`"

  @v0_3_0
  Scenario: A container that a macro_rules! writes dispatches the types the macro passes
    Then a container a macro_rules! writes should answer from "disk", and its factory field from "fake"

  @v0_3_0
  Scenario: Neither rustc nor clippy warns about the code snapfit writes for a dispatched type, however private its implementations are
    Given a container fixture "static_dispatch_container" for clippy
    When the fixture is linted with clippy
    Then clippy should report no warning
