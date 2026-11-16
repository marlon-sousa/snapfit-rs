use super::*;
use crate::domain::{ContainerName, DependencyName, InstantiationStep, Lifecycle, TypeName};
use crate::ports::CodeEmitterPort;

fn make_plan_with_dep(name: &str, ty: &str) -> ConstructionPlan {
    let mut plan = ConstructionPlan::new(ContainerName::new("AppContainer").unwrap(), false);
    plan.ordered_steps = vec![InstantiationStep::new(
        DependencyName::new(name).unwrap(),
        TypeName::new(ty).unwrap(),
        "Database::new()",
        false,
        Lifecycle::Singleton,
    )];
    plan
}

#[test]
fn test_quote_emitter_adapter_emits_valid_rust_items() {
    let plan = make_plan_with_dep("db", "Arc<Database>");
    let emitter = QuoteEmitterAdapter::new();

    let tokens = emitter.emit_container(&plan);
    let code = tokens.to_string();

    assert!(code.contains("pub struct AppContainer"));
    assert!(code.contains("pub fn get_db (& self) -> Arc < Database >"));
    assert!(!code.contains("AppContainerKey"));
    assert!(!code.contains("get_dependency"));

    let parsed_file: syn::File =
        syn::parse2(tokens).expect("emitted tokens must be valid Rust syntax");
    assert_eq!(parsed_file.items.len(), 4);
}

#[test]
fn test_full_adapter_and_use_case_pipeline_emits_valid_syntax() {
    use crate::ports::ContainerParserPort;

    let input: syn::ItemStruct = syn::parse_quote! {
        #[container]
        pub struct AppContainer {
            #[singleton]
            config: Arc<Config>,

            #[singleton(deps = [config])]
            db: Arc<Database>,

            #[transient(deps = [config, db])]
            user_controller: UserController,
        }
    };

    let ir = crate::adapters::SynParserAdapter::new()
        .parse_container(&input)
        .unwrap();
    let plan = crate::use_cases::DeriveContainerUseCase::default()
        .execute(ir)
        .unwrap();
    let tokens = QuoteEmitterAdapter::new().emit_container(&plan);
    let code = tokens.to_string();

    assert!(code.contains(
        "pub fn get_user_controller (& self) -> UserController { let config = self . get_config () ; let db = self . get_db () ; UserController :: new (config . clone () , db . clone ()) }"
    ));

    let parsed_file: syn::File =
        syn::parse2(tokens).expect("end-to-end emitted tokens must parse as valid Rust");
    assert_eq!(parsed_file.items.len(), 4);
}

#[test]
fn test_quote_emitter_adapter_emits_derive_impls_without_duplicate_struct() {
    let plan = make_plan_with_dep("db", "Arc<Database>");
    let emitter = QuoteEmitterAdapter::new();

    let tokens = emitter.emit_derive_impls(&plan);
    let code = tokens.to_string();

    assert!(!code.contains("pub struct AppContainer"));
    assert!(code.contains("pub fn new () -> Self"));
    assert!(code.contains("pub fn get_db (& self) -> Arc < Database >"));

    let parsed_file: syn::File =
        syn::parse2(tokens).expect("derive impl tokens must be valid Rust syntax");
    assert_eq!(parsed_file.items.len(), 3);
}
