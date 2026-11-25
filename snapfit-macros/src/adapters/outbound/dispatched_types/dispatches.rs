//! Adapter: the check, in words, that a dispatched type implements its
//! trait, since rustc's own error would name snapfit's dispatch types: a
//! trait without `#[snapfit::static_dispatch]` gives the dispatched type no
//! impl, and one whose implementations disagree on an associated type gives
//! it an impl it cannot meet. Plays the role of its parent.

use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;

use super::trait_name;
use crate::adapters::syntax::DispatchSyntax;

/// The check, in words, that the dispatched type implements its trait.
pub(super) fn dispatches(syntax: &DispatchSyntax, dispatched: &syn::Ident) -> TokenStream {
    let DispatchSyntax { trait_path, name } = syntax;
    let trait_name = trait_name(trait_path);
    let name = name.unraw();
    let message = format!(
        "`{name}` cannot dispatch the trait `{trait_name}` that static_dispatch_for names, because the trait, or one of its supertraits, is not marked #[snapfit::static_dispatch], an implementation on its list does not implement it, or the implementations disagree on an associated type"
    );
    let label = format!("`{name}` does not implement `{trait_name}`");
    let note = format!(
        "mark `{trait_name}` with #[snapfit::static_dispatch] where it is defined; an implementation that does not implement it is reported where its field or list names it; and give each associated type of `{trait_name}` the same type in every implementation on the list"
    );
    quote! {
        #[diagnostic::on_unimplemented(message = #message, label = #label, note = #note)]
        trait __SnapfitDispatches {}
        #[diagnostic::do_not_recommend]
        impl<__SnapfitDispatched: ?::core::marker::Sized + #trait_path> __SnapfitDispatches
            for __SnapfitDispatched {}
        fn __snapfit_dispatches<__SnapfitDispatched: ?::core::marker::Sized + __SnapfitDispatches>() {}
        let _ = __snapfit_dispatches::<#dispatched>;
    }
}
