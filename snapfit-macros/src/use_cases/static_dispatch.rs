//! Policy: which fields hold a type the container dispatches statically, the
//! implementations each such type holds, and what its declarations may not do.

use std::collections::HashSet;
use std::iter;

use crate::domain::{
    ContainerDeclaration, DependencyDeclaration, Dispatch, DispatchExtension,
    DispatchedTypeDeclaration, DispatchedTypePlan, DomainError, Implementation,
    StaticDispatchProblem, TypeName, Variant, WrittenType,
};

#[cfg(test)]
mod tests;

/// The names the container's `static_dispatch_for` declare, read from them.
#[derive(Debug, Default)]
pub(crate) struct DispatchedNames(Vec<TypeName>);

impl<S, T> FromIterator<DispatchedTypeDeclaration<S, T>> for DispatchedNames {
    fn from_iter<I: IntoIterator<Item = DispatchedTypeDeclaration<S, T>>>(declarations: I) -> Self {
        Self(
            declarations
                .into_iter()
                .map(|declaration| declaration.name)
                .collect(),
        )
    }
}

impl DispatchedNames {
    /// The declared name a field's value is written as, if any, also through
    /// a path, which the dispatch rules refuse on their own.
    pub(crate) fn held_by<F, T>(
        &self,
        dependency: &DependencyDeclaration<F, T>,
    ) -> Option<&TypeName> {
        let name = match &dependency.written_type {
            WrittenType::Name(name) | WrittenType::Path { last: name, .. } => name,
            WrittenType::Other => return None,
        };
        self.0.iter().find(|declared| *declared == name)
    }
}

pub(crate) fn dispatch_of(dispatched: Option<&TypeName>) -> Dispatch {
    dispatched.map_or(Dispatch::None, |_| Dispatch::Static)
}

/// Every type the container dispatches that holds an implementation, each
/// once, under the first `static_dispatch_for` that declares it. Its
/// implementations are those its fields name, then its `extra`, then its
/// extensions', each once.
pub(crate) fn dispatched_types<C, F, S: Clone, T: Clone>(
    container: &ContainerDeclaration<C, F, S, T>,
) -> Vec<DispatchedTypePlan<S, T>> {
    first_declarations(container)
        .filter_map(|declaration| planned(container, declaration))
        .collect()
}

/// Every problem of the container's dispatch declarations, and of a field
/// that names a declared type through a path.
pub(crate) fn dispatch_violations<C, F, S, T: Clone>(
    container: &ContainerDeclaration<C, F, S, T>,
) -> Vec<DomainError> {
    let declared: Vec<_> = first_declarations(container).collect();
    let names: Vec<_> = declared
        .iter()
        .map(|declaration| declaration.name.clone())
        .collect();
    let repeated = container
        .dispatched_types
        .iter()
        .zip(repeats_from(container))
        .filter_map(repeated_name);
    let undeclared = container
        .extensions
        .iter()
        .filter(|extension| !names.contains(&extension.name))
        .map(|extension| undeclared(extension, &names));
    let empty = declared
        .into_iter()
        .filter(|declaration| variants(container, declaration).is_empty())
        .map(without_implementations);
    let paths = iter::repeat(&names)
        .zip(&container.dependencies)
        .filter_map(through_path);
    repeated
        .chain(undeclared)
        .chain(empty)
        .chain(paths)
        .map(DomainError::StaticDispatch)
        .collect()
}

fn first_declarations<C, F, S, T>(
    container: &ContainerDeclaration<C, F, S, T>,
) -> impl Iterator<Item = &DispatchedTypeDeclaration<S, T>> {
    container
        .dispatched_types
        .iter()
        .zip(repeats_from(container))
        .filter(|(_, seen)| !*seen)
        .map(|(declaration, _)| declaration)
}

/// For each declaration, whether a declaration before it has its name.
fn repeats_from<C, F, S, T>(container: &ContainerDeclaration<C, F, S, T>) -> Vec<bool> {
    let mut seen = HashSet::new();
    container
        .dispatched_types
        .iter()
        .map(|declaration| !seen.insert(&declaration.name))
        .collect()
}

fn planned<C, F, S: Clone, T: Clone>(
    container: &ContainerDeclaration<C, F, S, T>,
    declaration: &DispatchedTypeDeclaration<S, T>,
) -> Option<DispatchedTypePlan<S, T>> {
    let mut variants = variants(container, declaration).into_iter();
    let first = variants.next()?;
    Some(DispatchedTypePlan::new(
        declaration.syntax.clone(),
        first,
        variants.collect(),
    ))
}

/// The implementations a declared type holds, in order, each once by how it is written.
fn variants<C, F, S, T: Clone>(
    container: &ContainerDeclaration<C, F, S, T>,
    declaration: &DispatchedTypeDeclaration<S, T>,
) -> Vec<Variant<T>> {
    let name = &declaration.name;
    let fields = container
        .dependencies
        .iter()
        .filter(|dependency| dependency.written_type == WrittenType::Name(name.clone()))
        .filter_map(named_implementation);
    let extensions = container
        .extensions
        .iter()
        .filter(|extension| &extension.name == name)
        .flat_map(|extension| &extension.variants);
    let all = fields.chain(&declaration.extra).chain(extensions);
    let mut seen = HashSet::new();
    all.filter(|variant| seen.insert(&variant.name))
        .cloned()
        .collect()
}

fn named_implementation<F, T>(dependency: &DependencyDeclaration<F, T>) -> Option<&Variant<T>> {
    match &dependency.implementation {
        Implementation::Named(variant) => Some(variant),
        Implementation::OfField => None,
    }
}

fn repeated_name<S, T>(
    (declaration, seen): (&DispatchedTypeDeclaration<S, T>, bool),
) -> Option<StaticDispatchProblem> {
    seen.then(|| StaticDispatchProblem::RepeatedName {
        position: declaration.position,
        name: declaration.name.clone(),
    })
}

fn undeclared<T>(extension: &DispatchExtension<T>, declared: &[TypeName]) -> StaticDispatchProblem {
    StaticDispatchProblem::Undeclared {
        position: extension.position,
        name: extension.name.clone(),
        declared: declared.to_vec(),
    }
}

fn without_implementations<S, T>(
    declaration: &DispatchedTypeDeclaration<S, T>,
) -> StaticDispatchProblem {
    StaticDispatchProblem::WithoutImplementations {
        position: declaration.position,
        name: declaration.name.clone(),
    }
}

fn through_path<F, T>(
    (declared, dependency): (&Vec<TypeName>, &DependencyDeclaration<F, T>),
) -> Option<StaticDispatchProblem> {
    let WrittenType::Path { last, written } = &dependency.written_type else {
        return None;
    };
    declared
        .contains(last)
        .then(|| StaticDispatchProblem::ThroughPath {
            dependency: dependency.name.clone(),
            name: last.clone(),
            written: written.clone(),
        })
}
