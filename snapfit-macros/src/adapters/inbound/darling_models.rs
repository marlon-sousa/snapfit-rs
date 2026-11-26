//! Adapter: the attribute arguments as darling reads them, before they are checked.

use darling::FromMeta;
use darling::ast::NestedMeta;
use derive_new::new;
use proc_macro2::TokenStream;

use crate::domain::ArgumentProblem;

mod build_meta;
mod expressions;
mod fallible_meta;
mod implementation_meta;
mod mode_meta;
mod parent_meta;
mod prerequisites;
mod repeats;
mod runtime_meta;
#[cfg(test)]
mod tests;

pub(super) use build_meta::{BuildMeta, BuildWord, FunctionNameMeta};
pub(super) use expressions::single_word;
pub(super) use fallible_meta::FallibleMeta;
use implementation_meta::FieldSettings;
pub(super) use implementation_meta::{
    ImplementationEntry, ImplementationKey, ImplementationMeta, ImplementationValue,
};
pub(super) use mode_meta::ModeMeta;
pub(super) use parent_meta::ParentTypeMeta;
pub(super) use prerequisites::{PrerequisiteEntry, PrerequisiteList, PrerequisiteMeta};
pub(super) use runtime_meta::RuntimeMeta;

#[derive(Debug, Default, Clone, PartialEq, Eq, FromMeta)]
pub(super) struct ContainerAttrMeta {
    #[darling(default)]
    pub(super) mode: Option<ModeMeta>,
    #[darling(default)]
    pub(super) parent: Option<ParentTypeMeta>,
    #[darling(default)]
    pub(super) introspection: bool,
    #[darling(default)]
    pub(super) leaf_build: Option<BuildMeta>,
    #[darling(default, rename = "crate")]
    pub(super) runtime: Option<RuntimeMeta>,
}

impl ContainerAttrMeta {
    pub(super) fn from_tokens(tokens: TokenStream) -> Result<Self, UnreadableAttribute<Self>> {
        read_list(tokens)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, FromMeta)]
pub(super) struct DependencyAttrMeta {
    #[darling(default)]
    pub(super) deps: PrerequisiteList,
    #[darling(default, rename = "async")]
    pub(super) is_async: bool,
    #[darling(default)]
    pub(super) build: Option<BuildMeta>,
    #[darling(default)]
    pub(super) fallible: Option<FallibleMeta>,
    /// Filled by `FieldSettings`, never by darling.
    #[darling(skip)]
    pub(super) implementations: Vec<ImplementationMeta>,
    /// Never filled: `FieldSettings` takes every `implementation` out
    /// first. Without it, darling's help for a misspelt setting would leave
    /// `implementation` out of the settings it offers.
    #[darling(default, rename = "implementation")]
    implementation_name: Option<darling::util::Ignored>,
    /// Never filled, as `implementation_name`.
    #[darling(default, rename = "implementations")]
    implementations_name: Option<darling::util::Ignored>,
}

impl DependencyAttrMeta {
    pub(super) fn from_attribute(attr: &syn::Attribute) -> Result<Self, UnreadableAttribute<Self>> {
        match &attr.meta {
            syn::Meta::Path(_) => Ok(Self::default()),
            syn::Meta::List(list) => Self::from_tokens(list.tokens.clone()),
            syn::Meta::NameValue(_) => Err(UnreadableAttribute::new(ArgumentProblem::Assigned)),
        }
    }

    /// When darling refuses the rest, every `implementation` follows the
    /// rescued settings as one value, so that a repeated one is still refused.
    fn from_tokens(tokens: TokenStream) -> Result<Self, UnreadableAttribute<Self>> {
        let FieldSettings {
            settings,
            implementations,
        } = syn::parse2(tokens)
            .map_err(unreadable)
            .map_err(UnreadableAttribute::new)?;
        match read_nested::<Self>(&settings) {
            Ok(meta) => Ok(Self {
                implementations,
                ..meta
            }),
            Err(mut unreadable) => {
                if !implementations.is_empty() {
                    unreadable.settings.push(Self {
                        implementations,
                        ..Self::default()
                    });
                }
                Err(unreadable)
            }
        }
    }
}

/// An attribute refused, for darling's problem with its settings or for a
/// setting written again, with each of its settings that reads on its own,
/// whose problems the first refusal leaves out.
#[derive(new, Debug)]
pub(super) struct UnreadableAttribute<Settings> {
    pub(super) problem: ArgumentProblem,
    /// The refusal of each setting written again after the first.
    #[new(default)]
    pub(super) repeated: Vec<ArgumentProblem>,
    /// One value per readable setting, holding that setting alone, in the order written.
    #[new(default)]
    pub(super) settings: Vec<Settings>,
}

fn read_list<Settings: FromMeta>(
    tokens: TokenStream,
) -> Result<Settings, UnreadableAttribute<Settings>> {
    let nested = NestedMeta::parse_meta_list(tokens)
        .map_err(unreadable)
        .map_err(UnreadableAttribute::new)?;
    read_nested(&nested)
}

/// The settings, each name read once; a setting written again is refused
/// beside whatever else is wrong, and read alone, so that its own value is
/// checked too.
fn read_nested<Settings: FromMeta>(
    nested: &[NestedMeta],
) -> Result<Settings, UnreadableAttribute<Settings>> {
    let (kept, again) = repeats::without_repeats(nested);
    let (problems, read_again): (Vec<_>, Vec<_>) =
        again.iter().map(repeats::read_copy::<Settings>).unzip();
    let mut repeated = problems.into_iter().flatten();
    let read_again = read_again.into_iter().flatten();
    match (Settings::from_list(&kept), repeated.next()) {
        (Ok(settings), None) => Ok(settings),
        (Ok(settings), Some(first)) => Err(UnreadableAttribute {
            problem: first,
            repeated: repeated.collect(),
            settings: std::iter::once(settings).chain(read_again).collect(),
        }),
        (Err(err), first) => Err(UnreadableAttribute {
            problem: unreadable(err),
            repeated: first.into_iter().chain(repeated).collect(),
            settings: kept
                .iter()
                .filter_map(read_alone)
                .chain(read_again)
                .collect(),
        }),
    }
}

/// A setting darling read with the others, read as if it were the only one;
/// one that does not read even alone is already in darling's problem with
/// them. This holds only while every field of the settings is
/// `#[darling(default)]`.
fn read_alone<Settings: FromMeta>(setting: &NestedMeta) -> Option<Settings> {
    Settings::from_list(std::slice::from_ref(setting)).ok()
}

fn unreadable(err: impl std::fmt::Display) -> ArgumentProblem {
    ArgumentProblem::Unreadable(err.to_string())
}
