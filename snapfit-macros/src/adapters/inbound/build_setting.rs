//! Adapter: reads `build = ...` on a field and `leaf_build = ...` on the container.

use quote::ToTokens;

use super::darling_models::{BuildMeta, BuildWord, FunctionNameMeta};
use crate::adapters::syntax::BuilderSyntax;
use crate::domain::{
    BuilderMethods, Construction, Constructor, DependencyName, DomainError, FunctionName,
};

mod builder;
#[cfg(test)]
mod tests;

/// What a field's `build = ...` says, with the functions it names as the
/// user wrote them; all empty when the field writes no `build`.
#[derive(Debug, Default, PartialEq)]
pub(super) struct WrittenBuild {
    pub(super) construction: Option<Construction>,
    pub(super) constructor: Option<syn::Ident>,
    pub(super) builder: Option<BuilderSyntax>,
}

impl WrittenBuild {
    fn of(construction: Construction) -> Self {
        Self {
            construction: Some(construction),
            ..Self::default()
        }
    }

    fn named_constructor((name, ident): (FunctionName, syn::Ident)) -> Self {
        Self {
            constructor: Some(ident),
            ..Self::of(Construction::Constructor(Constructor::Named(name)))
        }
    }

    fn builder((methods, syntax): (BuilderMethods, BuilderSyntax)) -> Self {
        Self {
            builder: Some(syntax),
            ..Self::of(Construction::Builder(methods))
        }
    }
}

/// How a field's `build = ...` builds it.
pub(super) fn field_construction(
    dependency: &DependencyName,
    build: Option<BuildMeta>,
) -> Result<WrittenBuild, DomainError> {
    let name = dependency.clone();
    match build {
        None => Ok(WrittenBuild::default()),
        Some(BuildMeta::Word { word, written }) => mode(dependency, word, &written),
        Some(BuildMeta::UnknownWord(word)) => Err(DomainError::UnknownBuild {
            dependency: name,
            written: word.to_string(),
        }),
        Some(BuildMeta::Builder { word, settings }) => {
            builder::methods(dependency, &word, &settings).map(WrittenBuild::builder)
        }
        Some(BuildMeta::Constructor(function)) => {
            function_name(dependency, function).map(WrittenBuild::named_constructor)
        }
        Some(BuildMeta::Quoted(written)) => Err(DomainError::QuotedBuild {
            dependency: name,
            written,
        }),
        Some(BuildMeta::Invalid(written)) => Err(DomainError::NotABuild {
            dependency: name,
            written,
        }),
    }
}

/// `build = default`, `factory` or `builder`, whose default functions are
/// spelt at the word; `constructor` without its function's name is refused.
fn mode(
    dependency: &DependencyName,
    word: BuildWord,
    written: &syn::Ident,
) -> Result<WrittenBuild, DomainError> {
    match word {
        BuildWord::Default => Ok(WrittenBuild::of(Construction::Constructor(
            Constructor::Default,
        ))),
        BuildWord::Factory => Ok(WrittenBuild::of(Construction::Factory)),
        BuildWord::Builder => builder::methods(dependency, written, &[]).map(WrittenBuild::builder),
        BuildWord::Constructor => Err(DomainError::ConstructorWithoutName {
            dependency: dependency.clone(),
        }),
    }
}

fn function_name(
    dependency: &DependencyName,
    value: FunctionNameMeta,
) -> Result<(FunctionName, syn::Ident), DomainError> {
    let dependency = dependency.clone();
    match value {
        FunctionNameMeta::Bare(ident) => {
            FunctionName::new(ident.to_string()).map(|name| (name, ident))
        }
        FunctionNameMeta::Quoted(written) if syn::parse_str::<syn::Ident>(&written).is_ok() => {
            Err(DomainError::QuotedFunctionName {
                dependency,
                written,
            })
        }
        FunctionNameMeta::Quoted(written) | FunctionNameMeta::Invalid(written) => {
            Err(DomainError::NotAFunctionName {
                dependency,
                written,
            })
        }
    }
}

/// How `leaf_build = ...` builds a field with no `deps` and no `build` of its
/// own: `default`, or `constructor(...)`; leaving it out means `new`.
/// The constructor's name comes back as the user wrote it, when there is one.
pub(super) fn leaf_constructor(
    leaf_build: Option<BuildMeta>,
) -> Result<(Constructor, Option<syn::Ident>), DomainError> {
    match leaf_build {
        None => Ok((Constructor::New, None)),
        Some(BuildMeta::Word {
            word: BuildWord::Default,
            ..
        }) => Ok((Constructor::Default, None)),
        Some(BuildMeta::Constructor(FunctionNameMeta::Bare(ident))) => {
            FunctionName::new(ident.to_string()).map(|name| (Constructor::Named(name), Some(ident)))
        }
        Some(other) => Err(DomainError::InvalidLeafBuild {
            written: written(&other),
        }),
    }
}

/// The value as the user wrote it, for a message that quotes it.
fn written(build: &BuildMeta) -> String {
    match build {
        BuildMeta::Word { written, .. } | BuildMeta::UnknownWord(written) => written.to_string(),
        BuildMeta::Builder { settings, .. } => format!("builder({})", written_settings(settings)),
        BuildMeta::Constructor(function) => format!("constructor({})", written_function(function)),
        BuildMeta::Quoted(text) => format!("\"{text}\""),
        BuildMeta::Invalid(text) => text.clone(),
    }
}

fn written_function(function: &FunctionNameMeta) -> String {
    match function {
        FunctionNameMeta::Bare(ident) => ident.to_token_stream().to_string(),
        FunctionNameMeta::Quoted(text) => format!("\"{text}\""),
        FunctionNameMeta::Invalid(text) => text.clone(),
    }
}

fn written_settings(settings: &[syn::Expr]) -> String {
    settings
        .iter()
        .map(written_setting)
        .collect::<Vec<_>>()
        .join(", ")
}

fn written_setting(setting: &syn::Expr) -> String {
    setting.to_token_stream().to_string()
}
