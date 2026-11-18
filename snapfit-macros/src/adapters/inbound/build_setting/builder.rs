//! Adapter: reads `build = builder` and `build = builder(start = ..., finish = ...)`.

use super::super::darling_models::single_word;
use super::written_settings;
use crate::adapters::syntax::BuilderSyntax;
use crate::domain::{BuilderMethods, DependencyName, DomainError, FunctionName};

#[cfg(test)]
mod tests;

/// The functions the builder chain starts and finishes with: the ones the
/// settings name, and `builder` and `build`, spelt at the `builder` word, for
/// the ones they leave out. Every setting that is not `start` or `finish`
/// with a function's name, once each, is refused, all in one error.
pub(super) fn methods(
    dependency: &DependencyName,
    word: &syn::Ident,
    settings: &[syn::Expr],
) -> Result<(BuilderMethods, BuilderSyntax), DomainError> {
    let read = settings.iter().fold(Settings::default(), Settings::with);
    if !read.refused.is_empty() {
        return Err(DomainError::InvalidBuilderSetting {
            dependency: dependency.clone(),
            written: written_settings(&read.refused),
        });
    }
    let syntax = BuilderSyntax {
        start: read
            .start
            .unwrap_or_else(|| syn::Ident::new("builder", word.span())),
        finish: read
            .finish
            .unwrap_or_else(|| syn::Ident::new("build", word.span())),
    };
    let methods = BuilderMethods::new(
        FunctionName::new(syntax.start.to_string())?,
        FunctionName::new(syntax.finish.to_string())?,
    );
    Ok((methods, syntax))
}

/// The settings of `builder(...)` read so far.
#[derive(Default)]
struct Settings {
    start: Option<syn::Ident>,
    finish: Option<syn::Ident>,
    refused: Vec<syn::Expr>,
}

impl Settings {
    fn with(mut self, setting: &syn::Expr) -> Self {
        match function_setting(setting) {
            Some((key, function)) if key == "start" && self.start.is_none() => {
                self.start = Some(function);
            }
            Some((key, function)) if key == "finish" && self.finish.is_none() => {
                self.finish = Some(function);
            }
            _ => self.refused.push(setting.clone()),
        }
        self
    }
}

/// `key = function`, both single words.
fn function_setting(setting: &syn::Expr) -> Option<(&syn::Ident, syn::Ident)> {
    let syn::Expr::Assign(assign) = setting else {
        return None;
    };
    let key = single_word(&assign.left)?;
    let function = single_word(&assign.right)?;
    Some((key, function.clone()))
}
