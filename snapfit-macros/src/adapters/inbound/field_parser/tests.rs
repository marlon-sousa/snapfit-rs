use syn::parse_quote;

use super::*;

#[test]
fn test_parse_field_singleton_with_deps_and_async_constructor() {
    let field: syn::Field = parse_quote! {
        #[singleton(deps = [config], async, constructor = connect)]
        db: std::sync::Arc<Database>
    };

    let dep = parse_field(&field).unwrap();
    assert_eq!(dep.name.as_str(), "db");
    assert_eq!(dep.type_string.as_str(), "std::sync::Arc<Database>");
    assert_eq!(
        dep.prerequisites,
        vec![DependencyName::new("config").unwrap()]
    );
    assert_eq!(dep.lifecycle, Lifecycle::Singleton);
    assert!(dep.is_async);
    assert_eq!(dep.constructor_name, "connect");
    assert_eq!(dep.custom_builder, None);
    assert_eq!(dep.trait_dispatch, TraitDispatch::None);
}

#[test]
fn test_parse_field_transient_and_custom_builder() {
    let field: syn::Field = parse_quote! {
        #[transient(deps = [config, db], builder = build_controller)]
        controller: UserController
    };

    let dep = parse_field(&field).unwrap();
    assert_eq!(dep.name.as_str(), "controller");
    assert_eq!(dep.lifecycle, Lifecycle::Transient);
    assert_eq!(dep.custom_builder.as_deref(), Some("build_controller"));
    assert_eq!(dep.constructor_name, "new");
}

#[test]
fn test_parse_field_enum_dispatch_and_dyn_dispatch() {
    let enum_field: syn::Field = parse_quote! {
        #[dispatch(PostgresDb, MockDb)]
        db: Arc<dyn Send + Sync + Database>
    };
    let enum_dep = parse_field(&enum_field).unwrap();
    assert_eq!(enum_dep.lifecycle, Lifecycle::Singleton);
    assert_eq!(
        enum_dep.trait_dispatch,
        TraitDispatch::Enum {
            trait_name: "Database".to_string(),
            impls: vec!["PostgresDb".to_string(), "MockDb".to_string()],
        }
    );

    let bare_dyn: syn::Field = parse_quote! {
        #[singleton]
        plugin: Arc<dyn Database + Send + Sync>
    };
    assert_eq!(
        parse_field(&bare_dyn).unwrap_err().to_string(),
        "invalid configuration: field 'plugin' has a trait-object type, which has no constructor; build it with a builder such as builder = build_store (unstable feature)"
    );

    let dyn_field: syn::Field = parse_quote! {
        #[singleton(builder = build_plugin)]
        plugin: Arc<dyn Database + Send + Sync>
    };
    let dyn_dep = parse_field(&dyn_field).unwrap();
    assert_eq!(
        dyn_dep.trait_dispatch,
        TraitDispatch::Dyn {
            trait_type: "dyn Database + Send + Sync".to_string(),
        }
    );
}

#[test]
fn test_parse_field_rejects_invalid_attribute_combinations() {
    let unannotated: syn::Field = parse_quote!(config: Arc<Config>);
    assert!(parse_field(&unannotated).is_err());

    let dual_lifecycle: syn::Field = parse_quote! {
        #[singleton]
        #[transient]
        config: Arc<Config>
    };
    assert!(parse_field(&dual_lifecycle).is_err());

    let both_ctor_and_builder: syn::Field = parse_quote! {
        #[singleton(constructor = connect, builder = build_db)]
        db: Arc<Database>
    };
    assert!(parse_field(&both_ctor_and_builder).is_err());

    let blank_ctor: syn::Field = parse_quote! {
        #[singleton(constructor = "   ")]
        db: Arc<Database>
    };
    assert!(parse_field(&blank_ctor).is_err());

    let async_transient: syn::Field = parse_quote! {
        #[transient(async)]
        controller: UserController
    };
    assert!(parse_field(&async_transient).is_err());
}

#[test]
fn test_parse_field_rejects_conditional_fields_and_non_identifier_names() {
    let conditional: syn::Field = parse_quote! {
        #[cfg(test)]
        #[singleton]
        config: Arc<Config>
    };
    let err = parse_field(&conditional).unwrap_err();
    assert_eq!(err.subject(), Some(&DependencyName::new("config").unwrap()));
    assert!(err.to_string().contains("cannot use #[cfg] or #[cfg_attr]"));

    let call_syntax: syn::Field = parse_quote! {
        #[singleton(constructor = "connect()")]
        db: Arc<Database>
    };
    assert_eq!(
        parse_field(&call_syntax).unwrap_err().to_string(),
        "invalid configuration: constructor 'connect()' on field 'db' must be a function name, such as constructor = connect"
    );
}

#[test]
fn test_parse_field_wants_bare_function_names() {
    let quoted: syn::Field = parse_quote! {
        #[singleton(constructor = "connect")]
        db: Arc<Database>
    };
    assert_eq!(
        parse_field(&quoted).unwrap_err().to_string(),
        "invalid configuration: constructor \"connect\" on field 'db' must be written without quotes: constructor = connect"
    );

    for (field, shown) in [
        (
            parse_quote!(#[singleton(constructor = connect())] db: Arc<Database>),
            "connect ()",
        ),
        (
            parse_quote!(#[singleton(builder = Db::build)] db: Arc<Database>),
            "Db :: build",
        ),
    ] {
        let field: syn::Field = field;
        let message = parse_field(&field).unwrap_err().to_string();
        assert!(
            message.contains(&format!("'{shown}' on field 'db' must be a function name")),
            "{message}"
        );
    }
}
