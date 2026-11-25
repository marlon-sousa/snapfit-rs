//! Adapter: reads `#[snapfit::static_dispatch]` and the item it is on, through
//! syn, into a dispatch attribute.

mod associated_type;
mod self_use;
mod signature;

#[cfg(test)]
mod tests;

use proc_macro2::{TokenStream, TokenTree};
use syn::ext::IdentExt;

use crate::adapters::syntax::{DispatchDeclaration, TraitSyntax};
use crate::domain::{
    Annotated, ConstantName, DispatchArguments, DispatchAttribute, DomainError, DomainErrors,
    ItemPosition, MacroName, MethodName, PlacedItem, Safety, TraitDeclaration,
    TraitItemDeclaration, TraitName, TypeName,
};
use associated_type::associated_type_shape;
use signature::method_shape;

type ItemDeclaration = TraitItemDeclaration<syn::TraitItemFn, syn::TraitItemType>;
type Placed = PlacedItem<syn::TraitItemFn, syn::TraitItemType>;

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

/// `remote = ...`, whatever follows the `=`, is the setting snapfit plans;
/// anything else is one it does not take.
fn arguments(args: &TokenStream) -> DispatchArguments {
    let mut tokens = args.clone().into_iter();

    match (tokens.next(), tokens.next(), tokens.next()) {
        (None, _, _) => DispatchArguments::None,
        (Some(TokenTree::Ident(name)), Some(TokenTree::Punct(equals)), Some(_))
            if name == "remote" && equals.as_char() == '=' =>
        {
            DispatchArguments::Remote
        }
        _ => DispatchArguments::Other,
    }
}

fn trait_declaration(
    item: &syn::ItemTrait,
) -> Result<TraitDeclaration<TraitSyntax, syn::TraitItemFn, syn::TraitItemType>, DomainErrors> {
    let name = TraitName::new(item.ident.unraw().to_string()).map_err(DomainErrors::from);
    let items = DomainErrors::collect(item.items.iter().enumerate().filter_map(placed_item));
    let (name, items) = DomainErrors::both(name, items)?;
    let syntax = TraitSyntax {
        ident: item.ident.clone(),
        generics: item.generics.clone(),
    };
    let safety = item.unsafety.map_or(Safety::Safe, |_| Safety::Unsafe);
    Ok(TraitDeclaration::new(name, syntax, safety, items))
}

/// The item as the domain sees it, at its position among every item of the
/// trait; tokens syn cannot read as an item are left out, for rustc to
/// report in the trait itself.
fn placed_item((position, item): (usize, &syn::TraitItem)) -> Option<Result<Placed, DomainErrors>> {
    let declaration = match item {
        syn::TraitItem::Fn(method) => method_declaration(method),
        syn::TraitItem::Type(associated) => type_declaration(associated),
        syn::TraitItem::Const(constant) => ConstantName::new(constant.ident.unraw().to_string())
            .map(ItemDeclaration::AssociatedConst),
        syn::TraitItem::Macro(call) => macro_name(&call.mac).map(ItemDeclaration::Macro),
        _ => return None,
    };
    Some(
        declaration
            .map(|declaration| place(position, declaration))
            .map_err(DomainErrors::from),
    )
}

fn place(position: usize, declaration: ItemDeclaration) -> Placed {
    PlacedItem::new(ItemPosition::new(position), declaration)
}

fn method_declaration(method: &syn::TraitItemFn) -> Result<ItemDeclaration, DomainError> {
    let name = MethodName::new(method.sig.ident.unraw().to_string())?;
    Ok(ItemDeclaration::Method {
        name,
        syntax: method.clone(),
        shape: method_shape(&method.sig),
    })
}

fn type_declaration(associated: &syn::TraitItemType) -> Result<ItemDeclaration, DomainError> {
    let name = TypeName::new(associated.ident.unraw().to_string())?;
    Ok(ItemDeclaration::AssociatedType {
        name,
        syntax: associated.clone(),
        shape: associated_type_shape(associated),
    })
}

/// The last segment of the macro's path, as its call is written.
fn macro_name(call: &syn::Macro) -> Result<MacroName, DomainError> {
    let last = call
        .path
        .segments
        .last()
        .map(|segment| segment.ident.unraw().to_string());
    MacroName::new(last.unwrap_or_default())
}
