use quote::quote;
use syn::parse_quote;

use super::*;
use crate::domain::{DependencyName, Lifecycle, TraitDispatch};
use crate::ports::ContainerParserPort;

#[test]
fn test_syn_parser_adapter_parses_full_container_struct() {
    let input: syn::ItemStruct = parse_quote! {
        #[container(mode = "async", parent = RootContainer, introspection = true)]
        pub struct AppContainer {
            #[singleton]
            config: Arc<Config>,

            #[singleton(deps = [config], async, constructor = "connect")]
            db: Arc<Database>,

            #[singleton(deps = [config, db], builder = "build_auth")]
            auth: Arc<AuthService>,

            #[transient(deps = [db])]
            controller: UserController,

            #[dispatch(PostgresDb, MockDb)]
            store: Arc<dyn DataStore>,
        }
    };

    let parser = SynParserAdapter::new();
    let ir = parser.parse_container(&input).unwrap();

    assert_eq!(ir.name.as_str(), "AppContainer");
    assert!(ir.is_async);
    assert_eq!(
        ir.parent_type.as_ref().map(|t| t.as_str()),
        Some("RootContainer")
    );
    assert!(ir.introspection);
    assert_eq!(ir.dependencies.len(), 5);

    assert_eq!(ir.dependencies[0].name.as_str(), "config");
    assert_eq!(ir.dependencies[0].lifecycle, Lifecycle::Singleton);

    assert_eq!(ir.dependencies[1].name.as_str(), "db");
    assert!(ir.dependencies[1].is_async);
    assert_eq!(ir.dependencies[1].constructor_name, "connect");
    assert_eq!(
        ir.dependencies[1].prerequisites,
        vec![DependencyName::new("config").unwrap()]
    );

    assert_eq!(ir.dependencies[2].name.as_str(), "auth");
    assert_eq!(
        ir.dependencies[2].custom_builder.as_deref(),
        Some("build_auth")
    );

    assert_eq!(ir.dependencies[3].name.as_str(), "controller");
    assert_eq!(ir.dependencies[3].lifecycle, Lifecycle::Transient);

    assert_eq!(
        ir.dependencies[4].trait_dispatch,
        TraitDispatch::Enum {
            trait_name: "DataStore".to_string(),
            impls: vec!["PostgresDb".to_string(), "MockDb".to_string()],
        }
    );
}

#[test]
fn test_syn_parser_adapter_infers_async_and_supports_parse_with_args() {
    let input: syn::ItemStruct = parse_quote! {
        pub struct AsyncInferredContainer {
            #[singleton(async, constructor = "load")]
            config: Arc<Config>,
        }
    };

    let parser = SynParserAdapter::new();
    let ir = parser
        .parse_with_args(quote!(introspection = true), &input)
        .unwrap();

    assert!(ir.is_async);
    assert!(ir.introspection);
}

#[test]
fn test_syn_parser_adapter_parses_derive_input() {
    let input: syn::DeriveInput = parse_quote! {
        #[container]
        pub struct SimpleApp {
            #[singleton(deps = [config], constructor = "connect")]
            db: Arc<Database>,
        }
    };

    let ir = SynParserAdapter::new().parse_derive_input(&input).unwrap();
    assert_eq!(ir.name.as_str(), "SimpleApp");
    assert_eq!(ir.dependencies[0].name.as_str(), "db");
    assert_eq!(ir.dependencies[0].constructor_name, "connect");
}

#[test]
fn test_syn_parser_adapter_rejects_invalid_container_configurations() {
    let parser = SynParserAdapter::new();

    let invalid_mode: syn::ItemStruct = parse_quote! {
        #[container(mode = "threaded")]
        pub struct BadMode {
            #[singleton]
            config: Arc<Config>,
        }
    };
    assert!(parser.parse_container(&invalid_mode).is_err());

    let sync_with_async_dep: syn::ItemStruct = parse_quote! {
        #[container(mode = "sync")]
        pub struct ConflictMode {
            #[singleton(async)]
            config: Arc<Config>,
        }
    };
    assert!(parser.parse_container(&sync_with_async_dep).is_err());

    let tuple_struct: syn::ItemStruct = parse_quote! {
        #[container]
        pub struct TupleContainer(Arc<Config>);
    };
    assert!(parser.parse_container(&tuple_struct).is_err());

    let enum_derive: syn::DeriveInput = parse_quote! {
        pub enum NotAStruct { A }
    };
    assert!(parser.parse_derive_input(&enum_derive).is_err());
}
