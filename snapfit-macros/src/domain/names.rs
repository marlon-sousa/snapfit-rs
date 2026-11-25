//! Entity: the names a container or a dispatched trait declares, each never blank.

use std::marker::PhantomData;

use derive_more::Display;

use crate::domain::errors::{DeclarationProblem, DomainError};

#[cfg(test)]
mod tests;

/// What a [`Name`] names, so that a dependency's name and a function's name
/// cannot be passed one for the other.
pub(crate) trait NameKind {
    const LABEL: &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Dependency;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Container;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Function;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Setter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Method;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Constant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Macro;

impl NameKind for Dependency {
    const LABEL: &'static str = "dependency name";
}

impl NameKind for Container {
    const LABEL: &'static str = "container name";
}

impl NameKind for Function {
    const LABEL: &'static str = "function name";
}

impl NameKind for Type {
    const LABEL: &'static str = "type name";
}

impl NameKind for Setter {
    const LABEL: &'static str = "setter name";
}

impl NameKind for Trait {
    const LABEL: &'static str = "trait name";
}

impl NameKind for Method {
    const LABEL: &'static str = "method name";
}

impl NameKind for Constant {
    const LABEL: &'static str = "constant name";
}

impl NameKind for Macro {
    const LABEL: &'static str = "macro name";
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display)]
#[display("{text}")]
pub(crate) struct Name<K> {
    text: String,
    kind: PhantomData<K>,
}

/// A container field, which is also the dependency it declares.
pub(crate) type DependencyName = Name<Dependency>;
pub(crate) type ContainerName = Name<Container>;
/// A constructor function, without its path.
pub(crate) type FunctionName = Name<Function>;
pub(crate) type TypeName = Name<Type>;
/// A method of a builder that takes one dependency.
pub(crate) type SetterName = Name<Setter>;
pub(crate) type TraitName = Name<Trait>;
pub(crate) type MethodName = Name<Method>;
/// An associated const of a trait.
pub(crate) type ConstantName = Name<Constant>;
/// The macro a trait calls among its items, by the last segment of its path.
pub(crate) type MacroName = Name<Macro>;

impl DependencyName {
    /// The builder setter of the same name, which takes this dependency when
    /// no label names another.
    pub(crate) fn as_setter(&self) -> SetterName {
        Name {
            text: self.text.clone(),
            kind: PhantomData,
        }
    }
}

impl<K: NameKind> Name<K> {
    pub(crate) fn new(text: impl Into<String>) -> Result<Self, DomainError> {
        let text = text.into();
        if text.trim().is_empty() {
            Err(DeclarationProblem::BlankName { label: K::LABEL })?;
        }
        Ok(Self {
            text,
            kind: PhantomData,
        })
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text
    }

    /// The name without the `r#` of a raw identifier, as generated names spell it.
    pub(crate) fn without_raw_prefix(&self) -> &str {
        self.text.strip_prefix("r#").unwrap_or(&self.text)
    }
}
