use quote::quote;

use super::*;

#[test]
fn a_factory_field_adds_the_factories_trait_and_new_calls_it() {
    let code = compact(expand_attribute(
        quote!(),
        quote! {
            pub struct Shop {
                #[singleton]
                config: Arc<Config>,
                #[singleton(deps = [config], build = factory)]
                payments: Arc<dyn PaymentGateway>,
            }
        },
        FeatureGate::Stable,
    ));

    assert!(code.contains("pubtraitShopFactories{"), "{code}");
    assert!(code.contains("fnbuild_payments(inputs:ShopPaymentsInputs)->Arc<dynPaymentGateway>;"));
    assert!(code.contains("pubstructShopPaymentsInputs{"));
    assert!(code.contains(
        "let__snapfit_payments:Arc<dynPaymentGateway>=<SelfasShopFactories>::build_payments(ShopPaymentsInputs{config:__snapfit_config.clone()});"
    ));
}

#[test]
fn a_container_without_factories_generates_no_factories_trait() {
    let code = compact(expand_attribute(
        quote!(),
        app_container(),
        FeatureGate::Stable,
    ));

    assert!(!code.contains("Factories"));
    assert!(!code.contains("Inputs"));
}
