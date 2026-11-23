Feature: Swapping a field for another implementation of its trait

  As a Rust developer whose containers choose which implementation of a trait each one uses
  I want the services that depend on that field to keep their code when a container swaps it
  So that a swap changes the container that makes it, and no service

  @v0_2_0
  Scenario Outline: A field typed as the trait object is built by the implementation it names, and every service built from it takes the trait object
    When the desk that reads from <source> is opened
    Then its reader getter should return the trait object, reading from <source>
    And its report, built by new, should read from <source>
    And its digest, built by its builder, should read from <source>
    And its summary, built by a factory from the trait object, should read from <source>

    Examples:
      | source |
      | disk   |
      | memory |

  @v0_2_0
  Scenario: A container written by a macro_rules! builds the implementation the macro passes
    When the desk that a macro_rules! writes, reading from disk, is opened
    Then its reader getter should return the trait object, reading from disk
    And its report, built by new, should read from disk
    And its digest, built by its builder, should read from disk
    And its summary, built by a factory from the trait object, should read from disk

  @v0_2_0 @regression
  Scenario: Clippy does not lint the code snapfit writes for an implementation that is not Send and Sync
    Given a container fixture "implementation_not_send" for clippy
    When the fixture is linted with clippy
    Then clippy should report no warning

  @v0_2_0 @regression
  Scenario Outline: A container written by a macro_rules! builds the implementation it passes as tokens
    When the desks whose implementation a macro_rules! passes as tokens are opened
    Then the desk whose implementation is passed as "<form>" should read from disk

    Examples:
      | form           |
      | a token tree   |
      | an identifier  |
      | a whole struct |

  @v0_2_0
  Scenario Outline: An implementation is built the way the field's build, or the container's leaf_build, says
    When the desks whose readers are built from their implementation in every way are opened
    Then the reader <reader> should read from "<source>"

    Examples:
      | reader    | source                     |
      | opened    | shelf opened               |
      | defaulted | shelf by default           |
      | built     | shelf built by its builder |
      | checked   | shelf checked              |
      | leaf      | shelf by default           |

  @v0_2_0
  Scenario: A fallible implementation that fails is returned by try_new, naming its field
    When the desks whose readers are built from their implementation in every way are opened
    Then the locked desk should fail at its field reader, with the error "the shelf is locked" as its source

  @v0_2_0 @regression
  Scenario: An implementation that is neither Send nor Sync is built into the field's Arc
    When the desk whose reader is neither Send nor Sync is opened
    Then its local reader should read from disk

  @v0_2_0
  Scenario Outline: An implementation written in a way snapfit cannot follow is refused at its field
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error>"
    And the error should point at line 24 of the fixture
    And the compiler should report <errors> errors

    Examples:
      | fixture                      | errors | expected_error |
      | bare_implementation          | 1      | field 'reader': implementation needs a type; write the type the container builds for the field, such as implementation = StripeGateway |
      | parenthesised_implementation | 1      | field 'reader': 'implementation(DiskReader)' writes its type in parentheses; write it after an equals sign: implementation = DiskReader |
      | quoted_implementation        | 1      | field 'reader': implementation takes a type, not a string; write it unquoted: implementation = DiskReader |
      | implementation_impl_trait    | 1      | field 'reader': 'impl Reader' is not a type the container can build; write the type in full, such as implementation = StripeGateway |
      | implementation_inferred      | 1      | field 'reader': 'Arc<_>' is not a type the container can build; write the type in full, such as implementation = StripeGateway |
      | implementation_trait_object  | 1      | field 'reader': 'dyn Reader' is not a type the container can build; write the type in full, such as implementation = StripeGateway |
      | implementation_not_a_type    | 1      | field 'reader': '3' is not a type the container can build; write the type in full, such as implementation = StripeGateway |
      | repeated_implementation      | 2      | field 'reader': implementation is given more than once; give the field one type to build |

  @v0_2_0
  Scenario Outline: An implementation on a field that cannot be built from it is refused at the field
    Given an invalid container fixture "<fixture>"
    When the fixture is compiled
    Then compilation should fail with "<expected_error>"
    And the error should point at line 24 of the fixture
    And the compiler should report 1 errors

    Examples:
      | fixture                             | expected_error |
      | implementation_without_trait_object | field 'reader' names an implementation, but its type holds no trait object; write the trait object as the field's type, such as Arc<dyn PaymentGateway>, or remove implementation |
      | implementation_outside_arc          | field 'reader' names an implementation, but holds its trait object in something other than an Arc; write the field as an Arc of the trait object, such as Arc<dyn PaymentGateway>, or remove implementation and build the field with build = factory |
      | implementation_with_factory         | field 'reader' names an implementation and is built with build = factory, which returns the field itself; remove implementation, or remove build = factory so that the container builds the implementation |

  @v0_2_0
  Scenario: An implementation that does not implement the field's trait is a compile error
    Given an invalid container fixture "implementation_not_implemented"
    When the fixture is compiled
    Then compilation should fail with "the trait bound `Notebook: Reader` is not satisfied"
    And the error should point at line 32 of the fixture
    And the compiler should report 1 errors
