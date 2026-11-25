//! Adapter: the dispatch attribute being read, and the errors about it,
//! each pointing at it. Plays the role of its parent.

use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

use super::super::type_inspector::is_buildable;
use super::tokens::{Head, bracketed, compact};
use crate::domain::{
    AttributePosition, DispatchAttributeKind, DomainError, DomainErrors, StaticDispatchProblem,
    TypeName, Variant,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct Site {
    pub(super) position: AttributePosition,
    kind: DispatchAttributeKind,
}

impl Site {
    pub(super) fn new(position: AttributePosition, kind: DispatchAttributeKind) -> Self {
        Self { position, kind }
    }

    /// The trait and the name it declares.
    pub(super) fn named(self, head: Head) -> Result<(syn::Path, syn::Ident), DomainErrors> {
        let Head { trait_path, name } = head;
        let name = name.ok_or_else(|| {
            self.error(StaticDispatchProblem::WithoutName {
                position: self.position,
            })
        })?;
        Ok((trait_path, self.single_name(&name)?))
    }

    /// The name of a dispatched type, which is a single identifier.
    pub(super) fn single_name(self, path: &syn::Path) -> Result<syn::Ident, DomainErrors> {
        path.get_ident().cloned().ok_or_else(|| {
            self.error(StaticDispatchProblem::NameAsPath {
                position: self.position,
                kind: self.kind,
                written: compact(quote!(#path)),
            })
        })
    }

    /// Every type of a list in square brackets, or every entry that is no type.
    pub(super) fn type_list(
        self,
        value: TokenStream,
    ) -> Result<Vec<Variant<syn::Type>>, DomainErrors> {
        let inner = bracketed(value.clone()).ok_or_else(|| {
            self.error(StaticDispatchProblem::NotAList {
                position: self.position,
                kind: self.kind,
                written: compact(value),
            })
        })?;
        let types = Punctuated::<syn::Type, syn::Token![,]>::parse_terminated
            .parse2(inner)
            .map_err(|error| self.unreadable(error))?;
        DomainErrors::collect(types.into_iter().map(|ty| self.variant(ty)))
    }

    fn variant(self, ty: syn::Type) -> Result<Variant<syn::Type>, DomainErrors> {
        let written = compact(quote!(#ty));
        if !is_buildable(&ty) {
            return Err(self.error(StaticDispatchProblem::NotAType {
                position: self.position,
                kind: self.kind,
                written,
            }));
        }
        Ok(Variant::new(TypeName::new(written)?, ty))
    }

    pub(super) fn unreadable(self, problem: impl Display) -> DomainErrors {
        self.error(StaticDispatchProblem::Unreadable {
            position: self.position,
            kind: self.kind,
            problem: problem.to_string(),
        })
    }

    pub(super) fn unknown_setting(self, written: String) -> DomainErrors {
        self.error(StaticDispatchProblem::UnknownSetting {
            position: self.position,
            written,
        })
    }

    pub(super) fn repeated_extra(self) -> DomainErrors {
        self.error(StaticDispatchProblem::RepeatedExtra {
            position: self.position,
        })
    }

    pub(super) fn without_list(self, name: &TypeName) -> DomainErrors {
        self.error(StaticDispatchProblem::WithoutList {
            position: self.position,
            name: name.clone(),
        })
    }

    fn error(self, problem: StaticDispatchProblem) -> DomainErrors {
        DomainError::StaticDispatch(problem).into()
    }
}
