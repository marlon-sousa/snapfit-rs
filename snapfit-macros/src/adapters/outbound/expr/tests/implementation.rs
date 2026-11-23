use syn::parse_quote;

use super::built;
use crate::adapters::outbound::expr::ValueSource;

#[test]
fn an_arc_of_a_trait_object_is_built_by_the_implementation_it_names_and_wrapped() {
    let item: syn::ItemStruct = parse_quote! {
        struct App {
            #[singleton]
            config: Arc<Config>,
            #[singleton(implementation = DiskReader)]
            plain: Arc<dyn Reader>,
            #[singleton(deps = [config], implementation = DiskReader, build = constructor(open))]
            opened: Arc<dyn Reader + Send + Sync>,
            #[singleton(deps = [config], implementation = Cache<Order>, build = builder)]
            built: Arc<dyn Reader>,
        }
    };

    assert_eq!(
        built(item.clone(), "plain", ValueSource::Locals),
        "{let__snapfit_implementation=::std::sync::Arc::new(<DiskReader>::new());__snapfit_implementation}"
    );
    assert_eq!(
        built(item.clone(), "opened", ValueSource::Locals),
        "{let__snapfit_implementation=::std::sync::Arc::new(<DiskReader>::open(__snapfit_config.clone()));__snapfit_implementation}"
    );
    assert_eq!(
        built(item, "built", ValueSource::Locals),
        "{let__snapfit_implementation=::std::sync::Arc::new(<Cache<Order>>::builder().config(__snapfit_config.clone()).build());__snapfit_implementation}"
    );
}
