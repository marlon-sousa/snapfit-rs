//! Adapter: the settings of `#[container(...)]`, read and checked. Plays the
//! role of its parent.

use proc_macro2::TokenStream;

use super::darling_models::{ContainerAttrMeta, UnreadableAttribute};
use super::{build_setting, runtime_path};
use crate::adapters::syntax::is_container_attribute;
use crate::domain::{
    ArgumentProblem, AttributeProblem, Constructor, DeclarationProblem, DomainError, DomainErrors,
    Execution, TypeName,
};

/// The settings of `#[container(...)]` that are checked when they are read.
pub(super) struct ContainerSettings {
    pub(super) mode: Option<Execution>,
    pub(super) parent: Option<TypeName>,
    pub(super) leaf_constructor: Constructor,
    /// The function of `leaf_build = constructor(...)` as the user wrote it.
    pub(super) leaf_ident: Option<syn::Ident>,
    pub(super) runtime: syn::Path,
}

/// The container's `mode`, `parent` and `leaf_build`, or every problem they have.
/// Each setting must be checked apart from the others: one rescued from an
/// attribute that does not read is checked alone.
pub(super) fn container_settings(
    meta: ContainerAttrMeta,
) -> Result<ContainerSettings, DomainErrors> {
    let mode = meta
        .mode
        .as_deref()
        .map(execution_of)
        .transpose()
        .map_err(DomainErrors::from);
    let parent = meta
        .parent
        .map(|parent| TypeName::new(parent.0))
        .transpose()
        .map_err(DomainErrors::from);
    let leaf = build_setting::leaf_constructor(meta.leaf_build).map_err(DomainErrors::from);
    let runtime = runtime_path::runtime_path(meta.runtime).map_err(DomainErrors::from);
    let (mode, (parent, ((leaf_constructor, leaf_ident), runtime))) = DomainErrors::both(
        mode,
        DomainErrors::both(parent, DomainErrors::both(leaf, runtime)),
    )?;
    Ok(ContainerSettings {
        mode,
        parent,
        leaf_constructor,
        leaf_ident,
        runtime,
    })
}

/// `mode = "..."`, read by the adapter so that its error names the mode
/// without darling's location.
fn execution_of(mode: &str) -> Result<Execution, DomainError> {
    match mode {
        "sync" => Ok(Execution::Sync),
        "async" => Ok(Execution::Async),
        other => Err(AttributeProblem::Container {
            problem: ArgumentProblem::InvalidMode(other.to_string()),
        }
        .into()),
    }
}

/// The settings are the macro's own arguments; another `#[container]` left
/// on the struct is a second declaration.
pub(super) fn container_meta(
    args: TokenStream,
    attrs: &[syn::Attribute],
) -> Result<ContainerAttrMeta, DomainErrors> {
    if attrs.iter().any(is_container_attribute) {
        Err(DeclarationProblem::MultipleContainerAttributes)?;
    }
    ContainerAttrMeta::from_tokens(args).map_err(unreadable_container_attribute)
}

/// The attribute's problem, then the problems of each setting that reads on its own.
fn unreadable_container_attribute(
    unreadable: UnreadableAttribute<ContainerAttrMeta>,
) -> DomainErrors {
    let UnreadableAttribute { problem, settings } = unreadable;
    let found = settings.into_iter().map(container_settings);
    let rest = found.filter_map(Result::err).flatten();
    DomainErrors::new(
        AttributeProblem::Container { problem }.into(),
        rest.collect(),
    )
}
