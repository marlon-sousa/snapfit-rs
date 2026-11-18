use crate::unstable_containers::{
    AsyncShopContainer, LayeredShopContainer, TransientFactoryShopContainer, TransientShopContainer,
};

#[derive(Default)]
pub struct UnstableContainers {
    async_shop: Option<AsyncShopContainer>,
    transient_shop: Option<TransientShopContainer>,
    transient_factory_shop: Option<TransientFactoryShopContainer>,
    layered_shop: Option<LayeredShopContainer>,
}

impl UnstableContainers {
    pub async fn init_async_shop(&mut self) {
        let container = AsyncShopContainer::init()
            .await
            .unwrap_or_else(|err| panic!("AsyncShopContainer::init() failed: {err}"));
        self.async_shop = Some(container);
    }

    pub fn new_transient_shop(&mut self) {
        self.transient_shop = Some(TransientShopContainer::new());
    }

    pub fn new_transient_factory_shop(&mut self) {
        self.transient_factory_shop = Some(TransientFactoryShopContainer::new());
    }

    pub fn new_layered_shop(&mut self) {
        self.layered_shop = Some(LayeredShopContainer::new());
    }

    pub fn async_shop(&self) -> &AsyncShopContainer {
        self.async_shop
            .as_ref()
            .expect("AsyncShopContainer must be initialized before Then assertions")
    }

    pub fn transient_shop(&self) -> &TransientShopContainer {
        self.transient_shop
            .as_ref()
            .expect("TransientShopContainer must be instantiated before Then assertions")
    }

    pub fn transient_factory_shop(&self) -> &TransientFactoryShopContainer {
        self.transient_factory_shop
            .as_ref()
            .expect("TransientFactoryShopContainer must be instantiated before Then assertions")
    }

    pub fn layered_shop(&self) -> &LayeredShopContainer {
        self.layered_shop
            .as_ref()
            .expect("LayeredShopContainer must be instantiated before Then assertions")
    }
}
