use syn::parse_quote;

use super::*;
use crate::domain::DependencyName;

fn syntax_of(item: syn::ItemStruct) -> ContainerSyntax {
    ContainerSyntax::new(
        &item.ident,
        &item.vis,
        &item.attrs,
        &item.generics,
        &item.fields,
    )
}

#[test]
fn test_container_syntax_keeps_user_attributes_and_drops_snapfit_ones() {
    let syntax = syntax_of(parse_quote! {
        /// The application container.
        #[derive(Clone)]
        pub(crate) struct App {
            /// Shared configuration.
            #[singleton]
            #[allow(dead_code)]
            pub config: Arc<Config>,

            #[transient(deps = [config])]
            cart: Cart,
        }
    });

    assert_eq!(syntax.ident, "App");
    assert_eq!(syntax.attrs.len(), 2);
    assert!(matches!(syntax.vis, syn::Visibility::Restricted(_)));

    let config = syntax.field("config").unwrap();
    assert_eq!(config.attrs.len(), 2);
    assert!(
        config
            .attrs
            .iter()
            .all(|attr| !attr.path().is_ident("singleton"))
    );
    assert!(matches!(config.vis, syn::Visibility::Public(_)));

    let cart = syntax.field("cart").unwrap();
    assert!(cart.attrs.is_empty());
    assert!(syntax.field("missing").is_none());
}

#[test]
fn test_diagnostics_underline_the_snapfit_attribute_or_the_field_name() {
    let syntax = syntax_of(parse_quote! {
        struct App {
            #[allow(dead_code)]
            #[singleton(deps = [confog])]
            db: Arc<Database>,

            plain: u8,
        }
    });

    assert_eq!(
        syntax.field("db").unwrap().origin.to_string(),
        "# [singleton (deps = [confog])]"
    );
    assert_eq!(syntax.field("plain").unwrap().origin.to_string(), "plain");
}

#[test]
fn test_error_for_keeps_the_domain_message_for_field_and_container_errors() {
    let syntax = syntax_of(parse_quote! {
        struct App {
            #[singleton(deps = [confog])]
            db: Arc<Database>,
        }
    });
    let db = DependencyName::new("db").unwrap();

    let field_level = DomainError::invalid_dependency(&db, "field 'db' is broken");
    let unknown_field = DomainError::DuplicateDependency(DependencyName::new("nope").unwrap());
    let container_level = DomainError::InvalidConfiguration("bad".into());

    assert_eq!(
        syntax.error_for(field_level).to_string(),
        "invalid configuration: field 'db' is broken"
    );
    assert_eq!(
        syntax.error_for(unknown_field.clone()).to_string(),
        unknown_field.to_string()
    );
    assert_eq!(
        syntax.error_for(container_level).to_string(),
        "invalid configuration: bad"
    );
}
