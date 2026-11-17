use syn::parse_quote;

use super::*;
use crate::domain::{Construction, Constructor, FunctionName, TypeShape};

fn message(field: &syn::Field) -> String {
    let errors: Vec<_> = parse_field(field).unwrap_err().into_iter().collect();
    assert_eq!(errors.len(), 1, "{errors:?}");
    errors[0].to_string()
}

#[test]
fn a_singleton_reads_its_prerequisites_execution_and_constructor() {
    let field: syn::Field = parse_quote! {
        #[singleton(deps = [config], async, constructor = connect)]
        db: std::sync::Arc<Database>
    };

    let dependency = parse_field(&field).unwrap();

    assert_eq!(dependency.name.as_str(), "db");
    assert_eq!(
        dependency.prerequisites,
        vec![DependencyName::new("config").unwrap()]
    );
    assert_eq!(dependency.lifecycle, Lifecycle::Singleton);
    assert_eq!(dependency.execution, Execution::Async);
    assert_eq!(
        dependency.construction,
        Construction::Constructor(Constructor::Named(FunctionName::new("connect").unwrap()))
    );
    assert_eq!(dependency.shape, TypeShape::Arc);
    assert_eq!(dependency.dispatch, None);
}

#[test]
fn a_transient_reads_its_builder() {
    let field: syn::Field = parse_quote! {
        #[transient(deps = [config, db], builder = build_controller)]
        controller: UserController
    };

    let dependency = parse_field(&field).unwrap();

    assert_eq!(dependency.lifecycle, Lifecycle::Transient);
    assert_eq!(
        dependency.construction,
        Construction::Builder(FunctionName::new("build_controller").unwrap())
    );
    assert_eq!(dependency.shape, TypeShape::Value);
}

#[test]
fn a_dispatch_field_is_a_singleton_naming_its_trait_and_implementations() {
    let field: syn::Field = parse_quote! {
        #[dispatch(PostgresDb, MockDb)]
        db: Arc<dyn Send + Sync + Database>
    };

    let dependency = parse_field(&field).unwrap();

    assert_eq!(dependency.lifecycle, Lifecycle::Singleton);
    assert_eq!(
        dependency.dispatch,
        Some(EnumDispatch::new(
            TypeName::new("Database").unwrap(),
            vec![
                TypeName::new("PostgresDb").unwrap(),
                TypeName::new("MockDb").unwrap()
            ],
        ))
    );
}

#[test]
fn a_trait_object_field_is_reported_as_one_for_the_domain_to_judge() {
    let field: syn::Field = parse_quote! {
        #[singleton]
        plugin: Arc<dyn Database + Send + Sync>
    };

    assert_eq!(parse_field(&field).unwrap().shape, TypeShape::TraitObject);
}

#[test]
fn attributes_that_cannot_be_read_together_are_refused() {
    let cases: [(syn::Field, &str); 4] = [
        (
            parse_quote!(config: Arc<Config>),
            "field 'config' has no snapfit attribute; add #[singleton] to have the container build it",
        ),
        (
            parse_quote!(#[singleton] #[transient] config: Arc<Config>),
            "field 'config' cannot declare multiple lifecycle attributes",
        ),
        (
            parse_quote!(#[singleton(constructor = connect, builder = build_db)] db: Arc<Database>),
            "dependency 'db' cannot specify both 'constructor' and 'builder'",
        ),
        (
            parse_quote!(#[singleton(constructor = "   ")] db: Arc<Database>),
            "constructor '   ' on field 'db' must be a function name, such as constructor = connect",
        ),
    ];
    for (field, expected) in cases {
        assert_eq!(message(&field), expected);
    }
}

#[test]
fn a_conditional_field_is_refused_together_with_its_attribute_errors() {
    let field: syn::Field = parse_quote! {
        #[cfg(test)]
        #[singleton]
        #[transient]
        config: Arc<Config>
    };

    let errors: Vec<_> = parse_field(&field).unwrap_err().into_iter().collect();

    assert_eq!(errors.len(), 2);
    assert_eq!(
        errors[0].subject(),
        Some(&DependencyName::new("config").unwrap())
    );
    assert!(
        errors[0]
            .to_string()
            .contains("cannot use #[cfg] or #[cfg_attr]")
    );
}

#[test]
fn a_function_name_must_be_written_bare() {
    let quoted: syn::Field = parse_quote! {
        #[singleton(constructor = "connect")]
        db: Arc<Database>
    };
    assert_eq!(
        message(&quoted),
        "constructor \"connect\" on field 'db' must be written without quotes: constructor = connect"
    );

    let cases: [(syn::Field, &str); 3] = [
        (
            parse_quote!(#[singleton(constructor = "connect()")] db: Arc<Database>),
            "connect()",
        ),
        (
            parse_quote!(#[singleton(constructor = connect())] db: Arc<Database>),
            "connect ()",
        ),
        (
            parse_quote!(#[singleton(builder = Db::build)] db: Arc<Database>),
            "Db :: build",
        ),
    ];
    for (field, shown) in cases {
        let message = message(&field);
        assert!(
            message.contains(&format!("'{shown}' on field 'db' must be a function name")),
            "{message}"
        );
    }
}
