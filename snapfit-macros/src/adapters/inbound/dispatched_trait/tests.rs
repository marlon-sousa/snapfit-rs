use quote::quote;
use syn::parse_quote;

use super::*;

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
fn the_arguments_are_read_as_none_remote_or_other() {
    let item: syn::Item = parse_quote!(
        trait Database {}
    );
    let cases = [
        (quote!(), DispatchArguments::None),
        (quote!(remote = other::Store), DispatchArguments::Remote),
        (quote!(remote), DispatchArguments::Other),
        (quote!(inline), DispatchArguments::Other),
        (
            quote!(remote = other::Store, inline),
            DispatchArguments::Other,
        ),
    ];
    for (args, arguments) in cases {
        let written = args.to_string();
        assert_eq!(read(args, item.clone()).arguments, arguments, "{written}");
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

fn kind_of(item: &ItemDeclaration) -> String {
    match item {
        ItemDeclaration::Method { name, .. } => format!("method {name}"),
        ItemDeclaration::AssociatedType {
            name,
            parameters: TypeParameters::None,
            ..
        } => format!("type {name} without parameters"),
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
