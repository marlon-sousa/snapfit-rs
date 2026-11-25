//! Adapter: reads the container's `static_dispatch_for` and
//! `extend_static_dispatch`, through syn, into the declarations of the types
//! it dispatches statically.

mod site;
mod tokens;

use proc_macro2::TokenStream;
use syn::ext::IdentExt;

use super::type_inspector::compact_type_tokens;
use crate::adapters::syntax::{
    DispatchExtensionSyntax, DispatchSyntax, DispatchedTypeDeclarationSyntax, is_dispatch_attribute,
};
use crate::domain::{
    AttributePosition, DispatchAttributeKind, DomainErrors, TypeName, Unreadability,
};
use site::Site;
use tokens::{Head, chunks, setting_name};

type Declaration = DispatchedTypeDeclarationSyntax;
type Extension = DispatchExtensionSyntax;

/// Each `static_dispatch_for` and each `extend_static_dispatch`, in the
/// order written, or every problem any of them has.
pub(super) fn read_dispatch_declarations(
    attrs: &[syn::Attribute],
) -> Result<(Vec<Declaration>, Vec<Extension>), DomainErrors> {
    let (declarations, extensions) = by_kind(attrs);
    DomainErrors::both(
        DomainErrors::collect(declarations.into_iter().map(read_declaration)),
        DomainErrors::collect(extensions.into_iter().map(read_extension)),
    )
}

/// Each `static_dispatch_for` whose trait and name can be read, and each `extend_static_dispatch` that can
/// be read, leaving out the others.
pub(super) fn read_readable_dispatch_declarations(
    attrs: &[syn::Attribute],
) -> (Vec<Declaration>, Vec<Extension>) {
    let (declarations, extensions) = by_kind(attrs);
    (
        declarations
            .into_iter()
            .filter_map(readable_declaration)
            .collect(),
        extensions
            .into_iter()
            .map(read_extension)
            .filter_map(Result::ok)
            .collect(),
    )
}

type Placed<'a> = (usize, &'a syn::Attribute);

/// The declarations and the extensions, each with its position among the
/// attributes of either kind, which the diagnostics count the same way.
fn by_kind(attrs: &[syn::Attribute]) -> (Vec<Placed<'_>>, Vec<Placed<'_>>) {
    attrs
        .iter()
        .filter(|attr| is_dispatch_attribute(attr))
        .enumerate()
        .partition(is_declaration)
}

fn is_declaration((_, attr): &Placed<'_>) -> bool {
    attr.path().is_ident("static_dispatch_for")
}

fn read_declaration((index, attr): Placed<'_>) -> Result<Declaration, DomainErrors> {
    let site = Site::new(AttributePosition(index), DispatchAttributeKind::Declaration);
    let (first, rest) = split(site, attr)?;
    declaration(site, first, &rest)
}

fn readable_declaration(placed: Placed<'_>) -> Option<Declaration> {
    read_declaration(placed)
        .ok()
        .or_else(|| read_head(placed).ok())
}

/// The trait and the name alone, without the settings.
fn read_head((index, attr): Placed<'_>) -> Result<Declaration, DomainErrors> {
    let site = Site::new(AttributePosition(index), DispatchAttributeKind::Declaration);
    let (first, _) = split(site, attr)?;
    head(site, first)
}

fn read_extension((index, attr): Placed<'_>) -> Result<Extension, DomainErrors> {
    let site = Site::new(AttributePosition(index), DispatchAttributeKind::Extension);
    let (first, rest) = split(site, attr)?;
    extension(site, first, &rest)
}

/// The attribute's arguments up to the first top-level comma, and each
/// setting after it.
fn split(
    site: Site,
    attr: &syn::Attribute,
) -> Result<(TokenStream, Vec<TokenStream>), DomainErrors> {
    let mut chunks = chunks(arguments(site, attr)?).into_iter();
    let first = chunks.next().unwrap_or_default();
    Ok((first, chunks.collect()))
}

fn arguments(site: Site, attr: &syn::Attribute) -> Result<TokenStream, DomainErrors> {
    match &attr.meta {
        syn::Meta::List(list) => Ok(list.tokens.clone()),
        syn::Meta::Path(_) => Err(site.unreadable(Unreadability::WithoutParentheses)),
        syn::Meta::NameValue(_) => Err(site.unreadable(Unreadability::AfterEquals)),
    }
}

/// `<Trait> as <Name>`, with nothing after it.
fn declaration(
    site: Site,
    first: TokenStream,
    rest: &[TokenStream],
) -> Result<Declaration, DomainErrors> {
    let settings = DomainErrors::collect(rest.iter().map(|setting| unknown_setting(site, setting)));
    let (declaration, _) = DomainErrors::both(head(site, first), settings)?;
    Ok(declaration)
}

/// `<Trait> as <Name>`.
fn head(site: Site, first: TokenStream) -> Result<Declaration, DomainErrors> {
    let head = syn::parse2::<Head>(first).map_err(|error| site.syn_error(&error))?;
    let (trait_path, name) = site.named(head)?;
    let type_name = type_name_of(&name)?;
    Ok(Declaration::new(
        type_name,
        site.position,
        DispatchSyntax { trait_path, name },
    ))
}

fn type_name_of(ident: &syn::Ident) -> Result<TypeName, DomainErrors> {
    Ok(TypeName::new(ident.unraw().to_string())?)
}

/// A setting after the name, which `static_dispatch_for` takes none of.
fn unknown_setting(site: Site, setting: &TokenStream) -> Result<(), DomainErrors> {
    Err(match setting_name(setting) {
        Some((name, _)) => site.unknown_setting(name.to_string()),
        None => site.unknown_setting(compact_type_tokens(setting.clone())),
    })
}

/// `<Name>, [...]`.
fn extension(
    site: Site,
    first: TokenStream,
    rest: &[TokenStream],
) -> Result<Extension, DomainErrors> {
    let name = syn::parse2::<syn::Path>(first)
        .map_err(|error| site.syn_error(&error))
        .and_then(|path| site.single_name(&path));
    let type_name = name.and_then(|ident| type_name_of(&ident));
    let list =
        match rest {
            [] => Ok(None),
            [list] => site.type_list(list.clone()).map(Some),
            [_, second, ..] => Err(site.unreadable(Unreadability::SecondList(
                compact_type_tokens(second.clone()),
            ))),
        };
    let (type_name, list) = DomainErrors::both(type_name, list)?;
    let variants = list.ok_or_else(|| site.without_list(&type_name))?;
    Ok(Extension::new(type_name, site.position, variants))
}

#[cfg(test)]
mod tests;
