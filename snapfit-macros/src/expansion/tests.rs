use quote::quote;

use super::*;
use crate::test_support::compact;

mod dispatched;
mod factories;

/// The messages of the `compile_error!` calls in `tokens`, in order.
fn compile_errors(tokens: &TokenStream) -> Vec<String> {
    let file: syn::File = syn::parse2(tokens.clone()).unwrap();
    file.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Macro(item) if is_compile_error(&item.mac.path) => Some(
                syn::parse2::<syn::LitStr>(item.mac.tokens.clone())
                    .unwrap()
                    .value(),
            ),
            _ => None,
        })
        .collect()
}

fn is_compile_error(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "compile_error")
}

fn app_container() -> TokenStream {
    quote! {
        pub struct AppContainer {
            #[singleton]
            config: Arc<Config>,

            #[singleton(deps = [config])]
            db: Arc<Database>,
        }
    }
}

#[test]
fn the_attribute_writes_the_struct_its_constructor_and_its_getters() {
    let code = compact(expand_attribute(
        quote!(),
        app_container(),
        FeatureGate::Stable,
    ));

    assert!(code.contains("pubstructAppContainer"));
    assert!(code.contains("pubfnnew()->Self"));
    assert!(code.contains("pubfnget_config(&self)->Arc<Config>"));
    assert!(code.contains("pubfnget_db(&self)->Arc<Database>"));
    assert!(
        compile_errors(&expand_attribute(
            quote!(),
            app_container(),
            FeatureGate::Stable
        ))
        .is_empty()
    );
}

#[test]
fn an_async_container_needs_the_unstable_build() {
    let unstable = compact(expand_attribute(
        quote!(mode = "async"),
        app_container(),
        FeatureGate::Unstable,
    ));
    let stable = expand_attribute(quote!(mode = "async"), app_container(), FeatureGate::Stable);

    assert!(unstable.contains("pubasyncfninit()"));
    assert_eq!(
        compile_errors(&stable),
        [
            "#[container(mode = \"async\")] is unstable in this version of snapfit; enable the `unstable` feature of the `snapfit` crate to use it"
        ]
    );
}

#[test]
fn every_problem_is_reported_once_and_the_struct_is_kept() {
    let input = quote! {
        pub struct Broken {
            #[singleton(deps = [b])]
            a: Arc<ServiceA>,
            #[singleton(deps = [a])]
            b: Arc<ServiceB>,
            #[singleton(deps = [confog])]
            db: Arc<Database>,
            #[transient]
            cart: Cart,
        }
    };

    let tokens = expand_attribute(quote!(), input, FeatureGate::Stable);

    assert_eq!(
        compile_errors(&tokens),
        [
            "transient dependency 'cart' is unstable in this version of snapfit; enable the `unstable` feature of the `snapfit` crate to use it",
            "dependency cycle: a depends on b, which depends on a",
            "unknown dependency 'confog' on field 'db'. You must choose between the dependencies the container can wire: [a, b, cart]",
        ]
    );
    assert!(
        compact(&tokens).contains(
            "pubstructBroken{a:Arc<ServiceA>,b:Arc<ServiceB>,db:Arc<Database>,cart:Cart,}"
        )
    );
}

#[test]
fn a_generic_container_is_refused_until_designed() {
    let input = quote! {
        pub struct Repo<T: Clone> {
            #[singleton]
            store: T,
        }
    };

    assert_eq!(
        compile_errors(&expand_attribute(quote!(), input, FeatureGate::Unstable)),
        [
            "generic parameters on container 'Repo' is planned for a future version of snapfit and is not implemented yet"
        ]
    );
}

#[test]
fn input_that_is_not_a_struct_gets_syn_s_own_error() {
    let tokens = expand_attribute(
        quote!(),
        quote!(
            fn not_a_struct() {}
        ),
        FeatureGate::Stable,
    );

    assert_eq!(compile_errors(&tokens).len(), 1);
}

#[test]
fn a_label_is_refused_on_a_field_not_built_by_a_builder_beside_one_that_is() {
    let built = expand_attribute(
        quote!(),
        quote! {
            pub struct Site {
                #[singleton]
                folder_files: Arc<FolderFiles>,
                #[singleton(deps = [disk = folder_files], build = builder)]
                writer: Arc<SiteWriter>,
                #[singleton(deps = [disk = folder_files])]
                reader: Arc<SiteReader>,
            }
        },
        FeatureGate::Stable,
    );

    assert_eq!(
        compile_errors(&built),
        [
            "the dependency 'disk = folder_files' of field 'reader' has a label, which names a builder setter; write folder_files alone, or build the field with build = builder"
        ]
    );
}
