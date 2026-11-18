use syn::parse_quote;

use super::super::field_parser::parse_field;
use super::*;

fn message(field: &syn::Field) -> String {
    let errors: Vec<_> = parse_field(field).unwrap_err().into_iter().collect();
    assert_eq!(errors.len(), 1, "{errors:?}");
    errors[0].to_string()
}

fn leaf(build: syn::Expr) -> Result<Constructor, DomainError> {
    let attr: syn::Attribute = parse_quote!(#[container(leaf_build = #build)]);
    let syn::Meta::List(list) = attr.meta else {
        panic!("a list");
    };
    let meta = super::super::darling_models::ContainerAttrMeta::from_tokens(list.tokens).unwrap();
    leaf_constructor(meta.leaf_build)
}

#[test]
fn each_build_word_reads_as_its_way_of_building() {
    let cases: [(syn::Field, Option<Construction>); 4] = [
        (parse_quote!(#[singleton] db: Arc<Database>), None),
        (
            parse_quote!(#[singleton(build = default)] rates: Arc<TaxRates>),
            Some(Construction::Constructor(Constructor::Default)),
        ),
        (
            parse_quote!(#[transient(deps = [config, db], build = factory)] controller: Controller),
            Some(Construction::Factory),
        ),
        (
            parse_quote!(#[singleton(deps = [config], build = builder)] exporter: Arc<Exporter>),
            Some(Construction::Builder),
        ),
    ];
    for (field, construction) in cases {
        assert_eq!(parse_field(&field).unwrap().construction, construction);
    }
}

#[test]
fn a_named_constructor_keeps_the_user_s_own_token() {
    let field: syn::Field =
        parse_quote!(#[singleton(build = constructor(connect))] db: Arc<Database>);

    let dependency = parse_field(&field).unwrap();

    let written = dependency.syntax.constructor.unwrap();
    assert_eq!(written.to_string(), "connect");
}

#[test]
fn a_build_value_that_is_no_way_of_building_is_refused_with_what_to_write() {
    let cases: [(syn::Field, &str); 4] = [
        (
            parse_quote!(#[singleton(build = factroy)] db: Arc<Database>),
            "build = factroy on field 'db' is not a way to build a field; write build = default, build = factory, or build = constructor(factroy) to call a function of that name",
        ),
        (
            parse_quote!(#[singleton(build = "factory")] db: Arc<Database>),
            "build \"factory\" on field 'db' must be written without quotes, such as build = factory",
        ),
        (
            parse_quote!(#[singleton(build = Db::connect)] db: Arc<Database>),
            "build = Db :: connect on field 'db' is not a way to build a field; write build = default, build = factory, or build = constructor(connect)",
        ),
        (
            parse_quote!(#[singleton(build = constructor)] db: Arc<Database>),
            "build = constructor on field 'db' needs the function's name, such as build = constructor(connect)",
        ),
    ];
    for (field, expected) in cases {
        assert_eq!(message(&field), expected);
    }
}

#[test]
fn a_function_name_must_be_written_bare() {
    let quoted: syn::Field = parse_quote! {
        #[singleton(build = constructor("connect"))]
        db: Arc<Database>
    };
    assert_eq!(
        message(&quoted),
        "constructor \"connect\" on field 'db' must be written without quotes: build = constructor(connect)"
    );

    let cases: [(syn::Field, &str); 3] = [
        (
            parse_quote!(#[singleton(build = constructor("connect()"))] db: Arc<Database>),
            "connect()",
        ),
        (
            parse_quote!(#[singleton(build = constructor(connect()))] db: Arc<Database>),
            "connect ()",
        ),
        (
            parse_quote!(#[singleton(build = constructor(Db::connect))] db: Arc<Database>),
            "Db :: connect",
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

#[test]
fn leaf_build_takes_default_or_a_constructor_and_nothing_else() {
    assert_eq!(leaf_constructor(None), Ok(Constructor::New));
    assert_eq!(leaf(parse_quote!(default)), Ok(Constructor::Default));
    assert_eq!(
        leaf(parse_quote!(constructor(open))),
        Ok(Constructor::Named(FunctionName::new("open").unwrap()))
    );

    let refused = [
        (parse_quote!(factory), "factory"),
        (parse_quote!(builder), "builder"),
        (parse_quote!("default"), "\"default\""),
        (
            parse_quote!(constructor(Db::open)),
            "constructor(Db :: open)",
        ),
    ];
    for (build, written) in refused {
        assert_eq!(
            leaf(build),
            Err(DomainError::InvalidLeafBuild {
                written: written.to_string()
            })
        );
    }
}
