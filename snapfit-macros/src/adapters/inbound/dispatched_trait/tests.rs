use quote::quote;
use syn::parse_quote;

use super::*;

mod arguments;

fn read(args: TokenStream, item: syn::Item) -> DispatchDeclaration {
    read_dispatch_attribute(&args, &item).unwrap()
}

fn trait_of(
    attribute: DispatchDeclaration,
) -> TraitDeclaration<TraitSyntax, syn::TraitItemFn, syn::TraitItemType> {
    match attribute.annotated {
        Annotated::Trait(declaration) => declaration,
        Annotated::Other => panic!("the attribute is on a trait"),
    }
}

#[test]
fn an_item_that_is_not_a_trait_is_read_as_another_item() {
    let attribute = read(
        quote!(),
        parse_quote!(
            struct Database;
        ),
    );

    assert!(attribute.annotated.is_other());
}

#[test]
fn each_item_of_the_trait_is_read_in_order_with_its_kind() {
    let declaration = trait_of(read(
        quote!(),
        parse_quote!(
            pub unsafe trait Database {
                type Row;
                type Rows<'a>;
                const LIMIT: usize;
                fn find(&self) -> Self::Row;
                sources!();
            }
        ),
    ));

    assert_eq!(declaration.name.as_str(), "Database");
    assert_eq!(declaration.safety, Safety::Unsafe);
    let kinds: Vec<_> = declaration.items.iter().map(kind_of).collect();
    assert_eq!(
        kinds,
        [
            "type Row without parameters",
            "type Rows with parameters",
            "const LIMIT",
            "method find",
            "macro sources",
        ]
    );
}

fn kind_of(item: &PlacedItem<syn::TraitItemFn, syn::TraitItemType>) -> String {
    match &item.declaration {
        ItemDeclaration::Method { name, .. } => format!("method {name}"),
        ItemDeclaration::AssociatedType { name, shape, .. }
            if shape.parameters == crate::domain::TypeParameters::None =>
        {
            format!("type {name} without parameters")
        }
        ItemDeclaration::AssociatedType { name, .. } => format!("type {name} with parameters"),
        ItemDeclaration::AssociatedConst(name) => format!("const {name}"),
        ItemDeclaration::Macro(name) => format!("macro {name}"),
    }
}

#[test]
fn a_macro_call_is_named_by_the_last_segment_of_its_path() {
    let declaration = trait_of(read(
        quote!(),
        parse_quote!(
            trait Database {
                crate::items::sources!();
            }
        ),
    ));

    assert_eq!(
        declaration.items.iter().map(kind_of).collect::<Vec<_>>(),
        ["macro sources"]
    );
}

#[test]
fn self_in_the_trait_s_own_bounds_is_read_wherever_it_is_written() {
    let elsewhere: [syn::Item; 5] = [
        parse_quote!(
            trait Store<T>
            where
                T: Into<Self>,
            {
            }
        ),
        parse_quote!(
            trait Store<T: Into<Self>> {}
        ),
        parse_quote!(
            trait Ranked: Ordered<Self> {}
        ),
        parse_quote!(
            trait Twin
            where
                Self: PartialEq<Self>,
            {
            }
        ),
        parse_quote!(
            trait Pair
            where
                Option<Self>: Clone,
            {
            }
        ),
    ];
    let forwardable: [syn::Item; 3] = [
        parse_quote!(
            trait Store<T>: Send + Sync
            where
                T: Clone,
            {
            }
        ),
        parse_quote!(
            trait Cloned
            where
                Self: Clone + 'static,
            {
            }
        ),
        parse_quote!(
            trait Rows
            where
                Self::Row: Clone,
            {
                type Row;
            }
        ),
    ];

    let bounds = |item: &syn::Item| trait_of(read(quote!(), item.clone())).bounds;

    assert!(
        elsewhere
            .iter()
            .map(bounds)
            .all(|bounds| bounds.is_elsewhere())
    );
    assert!(
        forwardable
            .iter()
            .map(bounds)
            .all(|bounds| bounds.is_forwardable())
    );
}
