use quote::quote;
use syn::parse_quote;

use super::*;
use crate::domain::{
    Construction, Constructor, EnumDispatch, Execution, FunctionName, Lifecycle, TypeShape,
};
use crate::test_support::names;

#[test]
fn every_setting_and_field_is_read_into_the_declaration() {
    let input: syn::ItemStruct = parse_quote! {
        #[container(mode = "async", parent = RootContainer, introspection = true)]
        pub struct AppContainer {
            #[singleton]
            config: Arc<Config>,

            #[singleton(deps = [config], async, constructor = connect)]
            db: Arc<Database>,

            #[singleton(deps = [config, db, parent(audit)], builder = build_auth)]
            auth: Arc<AuthService>,

            #[transient(deps = [db])]
            controller: UserController,

            #[dispatch(PostgresDb, MockDb)]
            store: Arc<dyn DataStore>,
        }
    };

    let container = parse_attribute(quote!(), &input).unwrap();

    assert_eq!(container.name.as_str(), "AppContainer");
    assert_eq!(container.form, MacroForm::Attribute);
    assert_eq!(container.mode, Some(Execution::Async));
    assert_eq!(
        container.parent,
        Some(TypeName::new("RootContainer").unwrap())
    );
    assert_eq!(container.introspection, Introspection::On);
    assert!(container.syntax.attrs.is_empty());

    let [config, db, auth, controller, store] = container.dependencies.as_slice() else {
        panic!("five dependencies");
    };
    assert_eq!(config.shape, TypeShape::Arc);
    assert_eq!(config.construction, Construction::default());

    assert_eq!(db.execution, Execution::Async);
    assert_eq!(
        db.construction,
        Construction::Constructor(Constructor::Named(FunctionName::new("connect").unwrap()))
    );
    assert_eq!(db.prerequisites, names(&["config"]));

    assert_eq!(
        auth.construction,
        Construction::Builder(FunctionName::new("build_auth").unwrap())
    );
    assert_eq!(auth.prerequisites, names(&["config", "db"]));
    assert_eq!(auth.parent_prerequisites, names(&["audit"]));

    assert_eq!(controller.lifecycle, Lifecycle::Transient);
    assert_eq!(controller.shape, TypeShape::Value);

    assert_eq!(store.shape, TypeShape::TraitObject);
    assert_eq!(
        store.dispatch,
        Some(EnumDispatch::new(
            TypeName::new("DataStore").unwrap(),
            vec![
                TypeName::new("PostgresDb").unwrap(),
                TypeName::new("MockDb").unwrap()
            ],
        ))
    );
}

#[test]
fn the_attribute_s_arguments_are_its_settings() {
    let input: syn::ItemStruct = parse_quote! {
        pub struct AppContainer {
            #[singleton]
            config: Arc<Config>,
        }
    };

    let container = parse_attribute(quote!(mode = "sync"), &input).unwrap();

    assert_eq!(container.mode, Some(Execution::Sync));
}

#[test]
fn the_derive_keeps_its_form_and_reads_its_helper_attribute() {
    let input: syn::DeriveInput = parse_quote! {
        #[container(introspection = true)]
        pub struct SimpleApp {
            #[singleton(constructor = connect)]
            db: Arc<Database>,
        }
    };

    let container = parse_derive(&input).unwrap();

    assert_eq!(container.form, MacroForm::Derive);
    assert_eq!(container.introspection, Introspection::On);
    assert_eq!(container.dependencies.len(), 1);
}

#[test]
fn a_field_keeps_its_own_syntax_without_snapfit_attributes() {
    let input: syn::ItemStruct = parse_quote! {
        struct App {
            /// The configuration.
            #[singleton]
            pub(crate) config: std::sync::Arc<Config>,
        }
    };

    let container = parse_attribute(quote!(), &input).unwrap();

    let syntax = &container.dependencies[0].syntax;
    assert_eq!(syntax.ident, "config");
    assert_eq!(syntax.attrs.len(), 1);
    let (ty, inner, vis) = (&syntax.ty, &syntax.arc_inner, &syntax.vis);
    assert_eq!(quote!(#ty).to_string(), "std :: sync :: Arc < Config >");
    assert_eq!(quote!(#inner).to_string(), "Config");
    assert_eq!(quote!(#vis).to_string(), "pub (crate)");
}

#[test]
fn type_parameters_are_reported_for_the_domain_to_judge() {
    let input: syn::ItemStruct = parse_quote! {
        struct Repo<T> {
            #[singleton]
            store: T,
        }
    };

    let container = parse_attribute(quote!(), &input).unwrap();

    assert_eq!(container.type_parameters, TypeParameters::Declared);
}

mod errors;
