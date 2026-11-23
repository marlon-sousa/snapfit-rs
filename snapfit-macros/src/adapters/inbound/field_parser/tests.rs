use quote::format_ident;
use syn::parse_quote;

use super::*;
use crate::domain::{Construction, Constructor, FunctionName, Prerequisite, TypeShape};
use crate::test_support::{labelled, prerequisites};

fn message(field: &syn::Field) -> String {
    let errors: Vec<_> = parse_field(field).unwrap_err().into_iter().collect();
    assert_eq!(errors.len(), 1, "{errors:?}");
    errors[0].to_string()
}

#[test]
fn a_singleton_reads_its_prerequisites_execution_and_constructor() {
    let field: syn::Field = parse_quote! {
        #[singleton(deps = [config], async, build = constructor(connect))]
        db: std::sync::Arc<Database>
    };

    let dependency = parse_field(&field).unwrap();

    assert_eq!(dependency.name.as_str(), "db");
    assert_eq!(dependency.prerequisites, prerequisites(&["config"]));
    assert_eq!(dependency.syntax.setters, [format_ident!("config")]);
    assert_eq!(dependency.lifecycle, Lifecycle::Singleton);
    assert_eq!(dependency.execution, Execution::Async);
    assert_eq!(
        dependency.construction,
        Some(Construction::Constructor(Constructor::Named(
            FunctionName::new("connect").unwrap()
        )))
    );
    assert_eq!(dependency.shape, TypeShape::Arc);
    assert_eq!(dependency.dispatch, None);
}

#[test]
fn a_raw_identifier_is_read_without_its_prefix() {
    let field: syn::Field = parse_quote! {
        #[singleton(build = builder, deps = [r#type, r#loop = config, "r#mod"])]
        r#match: std::sync::Arc<Database>
    };

    let dependency = parse_field(&field).unwrap();

    assert_eq!(dependency.name.as_str(), "match");
    assert_eq!(
        dependency.prerequisites,
        [
            Prerequisite::new(DependencyName::new("type").unwrap()),
            labelled("config", "loop"),
            Prerequisite::new(DependencyName::new("mod").unwrap()),
        ]
    );
    assert_eq!(
        dependency.syntax.setters,
        [
            format_ident!("r#type"),
            format_ident!("r#loop"),
            format_ident!("r#mod")
        ]
    );
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
fn an_arc_of_a_trait_object_is_reported_as_one_for_the_domain_to_judge() {
    let field: syn::Field = parse_quote! {
        #[singleton]
        plugin: Arc<dyn Database + Send + Sync>
    };

    assert_eq!(
        parse_field(&field).unwrap().shape,
        TypeShape::ArcTraitObject
    );
}

#[test]
fn attributes_that_cannot_be_read_together_are_refused() {
    let cases: [(syn::Field, &str); 2] = [
        (
            parse_quote!(config: Arc<Config>),
            "field 'config' has no snapfit attribute; add #[singleton] to have the container build it",
        ),
        (
            parse_quote!(#[singleton] #[transient] config: Arc<Config>),
            "field 'config' cannot declare multiple lifecycle attributes",
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
fn a_label_that_is_not_a_single_name_is_refused_with_how_to_write_one() {
    let field: syn::Field = parse_quote! {
        #[singleton(deps = [disk = files::folder], build = builder)]
        writer: Arc<SiteWriter>
    };

    assert_eq!(
        message(&field),
        "field 'writer': the deps entry 'disk = files::folder' is not a label; write the setter, then '=', then the field, each a single name, such as disk = folder_files"
    );
}
