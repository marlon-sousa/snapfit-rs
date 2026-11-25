//! Adapter: the dispatch attribute being read, and the errors about it,
//! each pointing at it. Plays the role of its parent.

use derive_new::new;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

use super::super::type_inspector::compact_type_tokens;
use super::super::type_inspector::is_buildable;
use super::tokens::{Head, bracketed};
use crate::domain::{
    AttributePosition, DispatchAttributeKind, DomainErrors, StaticDispatchProblem, TypeName,
    Unreadability, Variant,
};

#[derive(new, Debug, Clone, Copy)]
pub(super) struct Site {
    pub(super) position: AttributePosition,
    kind: DispatchAttributeKind,
}

impl Site {
    /// The trait and the name it declares, with nothing after the name.
    pub(super) fn named(self, head: Head) -> Result<(syn::Path, syn::Ident), DomainErrors> {
        let Head {
            trait_path,
            name,
            rest,
        } = head;
        let name = name.ok_or_else(|| {
            DomainErrors::from(StaticDispatchProblem::WithoutName {
                position: self.position,
            })
        })?;
        let name = self.single_name(&name);
        let after = if rest.is_empty() {
            Ok(())
        } else {
            Err(self.unreadable(Unreadability::AfterName(compact_type_tokens(rest))))
        };
        let (name, ()) = DomainErrors::both(name, after)?;
        Ok((trait_path, name))
    }

    /// The name of a dispatched type, which is a single identifier.
    pub(super) fn single_name(self, path: &syn::Path) -> Result<syn::Ident, DomainErrors> {
        path.get_ident().cloned().ok_or_else(|| {
            DomainErrors::from(StaticDispatchProblem::NameAsPath {
                position: self.position,
                kind: self.kind,
                written: compact_type_tokens(quote!(#path)),
            })
        })
    }

    /// Every type of a list in square brackets, or every entry that is no type.
    pub(super) fn type_list(
        self,
        value: TokenStream,
    ) -> Result<Vec<Variant<syn::Type>>, DomainErrors> {
        let inner = bracketed(value.clone()).ok_or_else(|| {
            DomainErrors::from(StaticDispatchProblem::NotAList {
                position: self.position,
                kind: self.kind,
                written: compact_type_tokens(value),
            })
        })?;
        let types = Punctuated::<syn::Type, syn::Token![,]>::parse_terminated
            .parse2(inner)
            .map_err(|error| self.syn_error(&error))?;
        DomainErrors::collect(types.into_iter().map(|ty| self.variant(ty)))
    }

    fn variant(self, ty: syn::Type) -> Result<Variant<syn::Type>, DomainErrors> {
        let written = compact_type_tokens(quote!(#ty));
        if !is_buildable(&ty) {
            return Err(StaticDispatchProblem::NotAType {
                position: self.position,
                kind: self.kind,
                written,
            }
            .into());
        }
        Ok(Variant::new(TypeName::new(written)?, ty))
    }

    pub(super) fn unreadable(self, problem: Unreadability) -> DomainErrors {
        DomainErrors::from(StaticDispatchProblem::Unreadable {
            position: self.position,
            kind: self.kind,
            problem,
        })
    }

    pub(super) fn syn_error(self, error: &syn::Error) -> DomainErrors {
        self.unreadable(Unreadability::Syn(error.to_string()))
    }

    pub(super) fn unknown_setting(self, written: String) -> DomainErrors {
        DomainErrors::from(StaticDispatchProblem::UnknownSetting {
            position: self.position,
            written,
        })
    }

    pub(super) fn repeated_extra(self) -> DomainErrors {
        DomainErrors::from(StaticDispatchProblem::RepeatedExtra {
            position: self.position,
        })
    }

    pub(super) fn without_list(self, name: &TypeName) -> DomainErrors {
        DomainErrors::from(StaticDispatchProblem::WithoutList {
            position: self.position,
            name: name.clone(),
        })
    }
}
