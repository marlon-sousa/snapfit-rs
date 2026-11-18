//! Entity: the names a container declaration uses, each never blank.

use std::marker::PhantomData;

use derive_more::Display;

use crate::domain::errors::DomainError;

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

impl<K: NameKind> Name<K> {
    pub(crate) fn new(text: impl Into<String>) -> Result<Self, DomainError> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(DomainError::BlankName { label: K::LABEL });
        }
        Ok(Self {
            text,
            kind: PhantomData,
        })
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text
    }
}
