//! Adapter: reads `#[snapfit::static_dispatch]` and the item it is on, through
//! syn, into a dispatch attribute.

mod signature;

#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;

use crate::adapters::syntax::{DispatchDeclaration, TraitSyntax};
use crate::domain::{
    Annotated, ConstantName, DispatchArguments, DispatchAttribute, DomainError, DomainErrors,
    MacroName, MethodName, Safety, TraitDeclaration, TraitItemDeclaration, TraitName, TypeName,
    TypeParameters,
};
use signature::method_shape;

type ItemDeclaration = TraitItemDeclaration<syn::TraitItemFn, syn::TraitItemType>;

/// `#[snapfit::static_dispatch(args)]` on `item`.
pub(crate) fn read_dispatch_attribute(
    args: &TokenStream,
    item: &syn::Item,
) -> Result<DispatchDeclaration, DomainErrors> {
    let annotated = match item {
        syn::Item::Trait(item) => Annotated::Trait(trait_declaration(item)?),
        _ => Annotated::Other,
    };
    Ok(DispatchAttribute::new(arguments(args), annotated))
}

/// `remote = <path>` alone is a setting snapfit plans; anything else is one
/// it does not take.
fn arguments(args: &TokenStream) -> DispatchArguments {
    if args.is_empty() {
        return DispatchArguments::None;
    }
    match syn::parse2::<syn::MetaNameValue>(args.clone()) {
        Ok(meta) if meta.path.is_ident("remote") => DispatchArguments::Remote,
        _ => DispatchArguments::Other,
    }
}

fn trait_declaration(
    item: &syn::ItemTrait,
) -> Result<TraitDeclaration<TraitSyntax, syn::TraitItemFn, syn::TraitItemType>, DomainErrors> {
    let name = TraitName::new(item.ident.to_string()).map_err(DomainErrors::from);
    let items = DomainErrors::collect(item.items.iter().filter_map(item_declaration));
    let (name, items) = DomainErrors::both(name, items)?;
    let syntax = TraitSyntax {
        ident: item.ident.clone(),
        generics: item.generics.clone(),
    };
    let mut declaration = TraitDeclaration::new(name, syntax);
    declaration.safety = item.unsafety.map_or(Safety::Safe, |_| Safety::Unsafe);
    declaration.items = items;
    Ok(declaration)
}

/// The item as the domain sees it; tokens syn cannot read as an item are
/// left out, for rustc to report in the trait itself.
fn item_declaration(item: &syn::TraitItem) -> Option<Result<ItemDeclaration, DomainErrors>> {
    let declaration = match item {
        syn::TraitItem::Fn(method) => {
            MethodName::new(method.sig.ident.to_string()).map(|name| ItemDeclaration::Method {
                name,
                syntax: method.clone(),
                shape: method_shape(&method.sig),
            })
        }
        syn::TraitItem::Type(associated) => {
            TypeName::new(associated.ident.to_string()).map(|name| {
                ItemDeclaration::AssociatedType {
                    name,
                    syntax: associated.clone(),
                    parameters: parameters(&associated.generics),
                }
            })
        }
        syn::TraitItem::Const(constant) => {
            ConstantName::new(constant.ident.to_string()).map(ItemDeclaration::AssociatedConst)
        }
        syn::TraitItem::Macro(call) => macro_name(&call.mac).map(ItemDeclaration::Macro),
        _ => return None,
    };
    Some(declaration.map_err(DomainErrors::from))
}

fn parameters(generics: &syn::Generics) -> TypeParameters {
    if generics.params.is_empty() {
        TypeParameters::None
    } else {
        TypeParameters::Declared
    }
}

/// The last segment of the macro's path, as its call is written.
fn macro_name(call: &syn::Macro) -> Result<MacroName, DomainError> {
    let last = call
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string());
    MacroName::new(last.unwrap_or_default())
}
