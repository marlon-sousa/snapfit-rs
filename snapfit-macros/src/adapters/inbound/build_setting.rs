//! Adapter: reads `build = ...` on a field and `leaf_build = ...` on the container.

use quote::ToTokens;

use super::darling_models::{BuildMeta, FunctionNameMeta};
use crate::domain::{Construction, Constructor, DependencyName, DomainError, FunctionName};

#[cfg(test)]
mod tests;

/// How a field's `build = ...` builds it, with the constructor's name as the
/// user wrote it when there is one; `None` when the field writes no `build`.
pub(super) fn field_construction(
    dependency: &DependencyName,
    build: Option<BuildMeta>,
) -> Result<(Option<Construction>, Option<syn::Ident>), DomainError> {
    let name = dependency.clone();
    match build {
        None => Ok((None, None)),
        Some(BuildMeta::Word(word)) => mode(dependency, &word).map(without_name),
        Some(BuildMeta::Builder) => Ok(without_name(Construction::Builder)),
        Some(BuildMeta::Constructor(function)) => {
            function_name(dependency, function).map(named_constructor)
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

/// `build = default`, `factory` or `builder`; `constructor` without its
/// function's name, and any other word, are refused.
fn mode(dependency: &DependencyName, word: &syn::Ident) -> Result<Construction, DomainError> {
    let dependency = dependency.clone();
    match word.to_string().as_str() {
        "default" => Ok(Construction::Constructor(Constructor::Default)),
        "factory" => Ok(Construction::Factory),
        "builder" => Ok(Construction::Builder),
        "constructor" => Err(DomainError::ConstructorWithoutName { dependency }),
        written => Err(DomainError::UnknownBuild {
            dependency,
            written: written.to_string(),
        }),
    }
}

fn named_constructor(
    (name, ident): (FunctionName, syn::Ident),
) -> (Option<Construction>, Option<syn::Ident>) {
    (
        Some(Construction::Constructor(Constructor::Named(name))),
        Some(ident),
    )
}

fn without_name(construction: Construction) -> (Option<Construction>, Option<syn::Ident>) {
    (Some(construction), None)
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
        Some(BuildMeta::Word(word)) if word == "default" => Ok((Constructor::Default, None)),
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
        BuildMeta::Word(word) => word.to_string(),
        BuildMeta::Builder => "builder(...)".to_string(),
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
