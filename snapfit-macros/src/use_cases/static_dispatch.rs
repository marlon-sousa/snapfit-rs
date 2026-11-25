//! Policy: which fields hold a type the container dispatches statically, and
//! the implementations each such type holds.

use std::collections::HashSet;

use derive_new::new;

use crate::domain::{
    ContainerDeclaration, DependencyDeclaration, Dispatch, DispatchExtension,
    DispatchedTypeDeclaration, DispatchedTypeFallback, DispatchedTypePlan, TypeName, Variant,
    WrittenType,
};

mod violations;

pub(crate) use violations::dispatch_violations;

/// The names the container's `static_dispatch_for` declare, read from them.
#[derive(Debug, Default)]
pub(crate) struct DispatchedNames(Vec<TypeName>);

impl<S> FromIterator<DispatchedTypeDeclaration<S>> for DispatchedNames {
    fn from_iter<I: IntoIterator<Item = DispatchedTypeDeclaration<S>>>(declarations: I) -> Self {
        Self(declarations.into_iter().map(name_of).collect())
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

fn name_of<S>(declaration: DispatchedTypeDeclaration<S>) -> TypeName {
    declaration.name
}

pub(crate) fn dispatch_of(dispatched: Option<&TypeName>) -> Dispatch {
    dispatched.map_or(Dispatch::None, |_| Dispatch::Static)
}

/// Every type the container dispatches that holds an implementation, each
/// once, under the first `static_dispatch_for` that declares it. Its
/// implementations are those its fields name, then its extensions', each once.
pub(crate) fn dispatched_types<C, F, S: Clone, T: Clone>(
    container: &ContainerDeclaration<C, F, S, T>,
) -> Vec<DispatchedTypePlan<S, T>> {
    let held = Held::new(&container.dependencies, &container.extensions);
    sorted(&container.dispatched_types)
        .first
        .into_iter()
        .filter_map(|declaration| held.planned(declaration))
        .collect()
}

/// For a container that is refused, each type its readable declarations
/// name, with the implementations the readable parts give it, or with none.
pub(crate) fn fallback_dispatched_types<F, S: Clone, T: Clone>(
    declarations: &[DispatchedTypeDeclaration<S>],
    extensions: &[DispatchExtension<T>],
    dependencies: &[DependencyDeclaration<F, T>],
) -> Vec<DispatchedTypeFallback<S, T>> {
    let held = Held::new(dependencies, extensions);
    sorted(declarations)
        .first
        .into_iter()
        .map(|declaration| held.fallback(declaration))
        .collect()
}

/// The declarations, the first of each name apart from those that repeat it.
#[derive(new)]
struct Sorted<'a, S> {
    #[new(default)]
    seen: HashSet<&'a TypeName>,
    #[new(default)]
    first: Vec<&'a DispatchedTypeDeclaration<S>>,
    #[new(default)]
    repeated: Vec<&'a DispatchedTypeDeclaration<S>>,
}

fn sorted<S>(declarations: &[DispatchedTypeDeclaration<S>]) -> Sorted<'_, S> {
    declarations.iter().fold(Sorted::new(), Sorted::with)
}

impl<'a, S> Sorted<'a, S> {
    fn with(mut self, declaration: &'a DispatchedTypeDeclaration<S>) -> Self {
        if self.seen.insert(&declaration.name) {
            self.first.push(declaration);
        } else {
            self.repeated.push(declaration);
        }
        self
    }
}

/// What gives a declared type its implementations.
#[derive(new)]
struct Held<'a, F, T> {
    dependencies: &'a [DependencyDeclaration<F, T>],
    extensions: &'a [DispatchExtension<T>],
}

impl<F, T: Clone> Held<'_, F, T> {
    fn planned<S: Clone>(
        &self,
        declaration: &DispatchedTypeDeclaration<S>,
    ) -> Option<DispatchedTypePlan<S, T>> {
        let mut variants = self.variants(declaration).into_iter();
        let first = variants.next()?;
        Some(DispatchedTypePlan::new(
            declaration.syntax.clone(),
            first,
            variants.collect(),
        ))
    }

    fn fallback<S: Clone>(
        &self,
        declaration: &DispatchedTypeDeclaration<S>,
    ) -> DispatchedTypeFallback<S, T> {
        match self.planned(declaration) {
            Some(planned) => DispatchedTypeFallback::Planned(planned),
            None => DispatchedTypeFallback::Unplanned(declaration.syntax.clone()),
        }
    }

    /// The implementations a declared type holds, in order, each once by how it is written.
    fn variants<S>(&self, declaration: &DispatchedTypeDeclaration<S>) -> Vec<Variant<T>> {
        let name = &declaration.name;
        let fields = self
            .dependencies
            .iter()
            .filter(|dependency| dependency.written_type.is_written_as(name))
            .flat_map(named_implementations);
        let extensions = self
            .extensions
            .iter()
            .filter(|extension| &extension.name == name)
            .flat_map(|extension| &extension.variants);
        let mut seen = HashSet::new();
        fields
            .chain(extensions)
            .filter(|variant| seen.insert(&variant.name))
            .cloned()
            .collect()
    }
}

fn named_implementations<F, T>(dependency: &DependencyDeclaration<F, T>) -> &[Variant<T>] {
    dependency.implementation.variants()
}

#[cfg(test)]
mod tests;
