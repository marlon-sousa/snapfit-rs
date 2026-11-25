//! Error: the parts of an error worded by the error that holds them.

use derive_more::Display;

use crate::domain::names::{ContainerName, DependencyName};

/// A feature this build of snapfit refuses, as its diagnostic names it.
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum Feature {
    #[display("#[container(parent = ...)]")]
    ParentContainer,
    #[display("#[container(introspection = true)]")]
    Introspection,
    #[display("generic parameters on container '{container}'")]
    GenericContainer { container: ContainerName },
    #[display("parent dependency 'parent({parent})' on dependency '{dependency}'")]
    ParentDependency {
        parent: DependencyName,
        dependency: DependencyName,
    },
    #[display("#[container(mode = \"async\")]")]
    AsyncMode,
    #[display("async dependency '{dependency}'")]
    AsyncDependency { dependency: DependencyName },
    #[display("transient dependency '{dependency}'")]
    Transient { dependency: DependencyName },
    #[display("build = factory on async dependency '{dependency}'")]
    AsyncFactory { dependency: DependencyName },
    #[display("build = builder on async dependency '{dependency}'")]
    AsyncBuilder { dependency: DependencyName },
    #[display("fallible = true on transient dependency '{dependency}'")]
    FallibleTransient { dependency: DependencyName },
    #[display("#[snapfit::static_dispatch(remote = ...)]")]
    RemoteStaticDispatch,
}

/// What would build a field marked `fallible = true` that cannot fail, with
/// the way out its error offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub(crate) enum InfallibleBuild {
    #[display(
        "build = default, which cannot fail; remove fallible = true, or build it another way"
    )]
    Default,
    #[display(
        "the container's leaf_build, since it has no deps and no build of its own, and leaf_build cannot fail; give the field a build of its own, such as build = constructor(new), or remove fallible = true"
    )]
    LeafBuild,
}

impl Feature {
    pub(crate) fn dependency(&self) -> Option<&DependencyName> {
        match self {
            Self::ParentDependency { dependency, .. }
            | Self::AsyncDependency { dependency }
            | Self::Transient { dependency }
            | Self::AsyncFactory { dependency }
            | Self::AsyncBuilder { dependency }
            | Self::FallibleTransient { dependency } => Some(dependency),
            Self::ParentContainer
            | Self::Introspection
            | Self::GenericContainer { .. }
            | Self::AsyncMode
            | Self::RemoteStaticDispatch => None,
        }
    }
}

/// What is wrong with an attribute's arguments, apart from which attribute.
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum AttributeProblem {
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
    #[display("implementation is given more than once; give the field one type to build")]
    RepeatedImplementation,
    #[display("invalid container mode '{_0}': write mode = \"sync\", or leave mode out")]
    InvalidMode(String),
}
