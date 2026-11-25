//! Error: what is wrong with an attribute's arguments, apart from which attribute.

use derive_more::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum ArgumentProblem {
    /// The words of darling or syn, which read the arguments.
    #[display("{_0}")]
    Unreadable(String),
    #[display("a name in the attribute cannot be blank")]
    BlankName,
    #[display("snapfit's attributes take their settings in parentheses, not after '='")]
    Assigned,
    #[display(
        "'{_0}' is not a list of fields; write the fields in square brackets, such as deps = [config]"
    )]
    DepsNotAList(String),
    /// The entries of `deps(...)`, as written between its parentheses.
    #[display(
        "'deps({_0})' writes its fields in parentheses; write them in square brackets: deps = [{_0}]"
    )]
    DepsInParentheses(String),
    #[display(
        "the deps entry '{_0}' is not the name of a field; write another field's name, such as config"
    )]
    NotAField(String),
    #[display(
        "the deps entry '{_0}' is not a label; write the setter, then '=', then the field, each a single name, such as disk = folder_files"
    )]
    NotALabel(String),
    #[display(
        "the deps entry '{_0}' names no single dependency; parent(...) is planned for a future version of snapfit and is not implemented yet"
    )]
    NotAParentDependency(String),
    #[display(
        "fallible needs a value; write fallible = true to have the container return the field's error, or leave fallible out"
    )]
    BareFallible,
    #[display(
        "fallible takes its value after '=', not in parentheses; write fallible = true, or fallible = false"
    )]
    FallibleInParentheses,
    /// The value as written, such as `"yes"`.
    #[display("fallible takes true or false, not {_0}; write fallible = true, or fallible = false")]
    NotAFallibleValue(String),
    #[display(
        "implementation needs a type; write the type the container builds for the field, such as implementation = StripeGateway"
    )]
    BareImplementation,
    /// The type as written between the parentheses.
    #[display(
        "'implementation({_0})' writes its type in parentheses; write it after an equals sign: implementation = {_0}"
    )]
    ImplementationInParentheses(String),
    /// The string's own text, such as `DiskReader`.
    #[display(
        "implementation takes a type, not a string; write it unquoted: implementation = {_0}"
    )]
    QuotedImplementation(String),
    /// The value as written, such as `3`, `_` or `impl Reader`.
    #[display(
        "'{_0}' is not a type the container can build; write the type in full, such as implementation = StripeGateway"
    )]
    NotAnImplementationType(String),
    #[display(
        "implementation = [] lists no type; list the implementations the field's factory may return, such as implementation = [Postgres, Sqlite], or remove implementation"
    )]
    EmptyImplementationList,
    #[display("implementation is given more than once; give the field one type to build")]
    RepeatedImplementation,
    #[display("invalid container mode '{_0}': write mode = \"sync\", or leave mode out")]
    InvalidMode(String),
}
