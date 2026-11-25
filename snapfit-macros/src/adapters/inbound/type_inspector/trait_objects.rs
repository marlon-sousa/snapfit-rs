//! Adapter: where a syn type holds `dyn Trait`, for the type inspector.

/// Where a type holds `dyn Trait`: as itself, as one of its own type
/// arguments, only deeper, or nowhere.
pub(super) enum Depth {
    Itself,
    Argument,
    Deeper,
    Nowhere,
}

pub(super) fn trait_object_depth(ty: &syn::Type) -> Depth {
    if is_trait_object(ty) {
        Depth::Itself
    } else if type_arguments(ty).into_iter().any(is_trait_object) {
        Depth::Argument
    } else if find_trait_object(ty).is_some() {
        Depth::Deeper
    } else {
        Depth::Nowhere
    }
}

/// The first `dyn Trait` in `ty`, itself or at any depth of its type
/// arguments.
pub(super) fn find_trait_object(ty: &syn::Type) -> Option<&syn::TypeTraitObject> {
    match ty {
        syn::Type::TraitObject(trait_object) => Some(trait_object),
        syn::Type::Group(group) => find_trait_object(&group.elem),
        syn::Type::Paren(paren) => find_trait_object(&paren.elem),
        _ => type_arguments(ty).into_iter().find_map(find_trait_object),
    }
}

/// `dyn Trait` itself, also when a macro or parentheses wrap it.
fn is_trait_object(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::TraitObject(_) => true,
        syn::Type::Group(group) => is_trait_object(&group.elem),
        syn::Type::Paren(paren) => is_trait_object(&paren.elem),
        _ => false,
    }
}

/// The type arguments of the last segment of `ty`'s path, or what a
/// reference points to, also when a macro or parentheses wrap `ty`.
fn type_arguments(ty: &syn::Type) -> Vec<&syn::Type> {
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(angle_bracketed_types)
            .unwrap_or_default(),
        syn::Type::Reference(reference) => vec![reference.elem.as_ref()],
        syn::Type::Group(group) => type_arguments(&group.elem),
        syn::Type::Paren(paren) => type_arguments(&paren.elem),
        _ => Vec::new(),
    }
}

fn angle_bracketed_types(segment: &syn::PathSegment) -> Vec<&syn::Type> {
    match &segment.arguments {
        syn::PathArguments::AngleBracketed(arguments) => {
            arguments.args.iter().filter_map(generic_type).collect()
        }
        _ => Vec::new(),
    }
}

fn generic_type(argument: &syn::GenericArgument) -> Option<&syn::Type> {
    match argument {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}
