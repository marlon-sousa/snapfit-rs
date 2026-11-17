use crate::unstable_containers::{
    AsyncShopContainer, BuilderShopContainer, LayeredShopContainer, TransientShopContainer,
};

#[derive(Default)]
pub struct UnstableContainers {
    async_shop: Option<AsyncShopContainer>,
    transient_shop: Option<TransientShopContainer>,
    builder_shop: Option<BuilderShopContainer>,
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

    pub fn new_builder_shop(&mut self) {
        self.builder_shop = Some(BuilderShopContainer::new());
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

    pub fn builder_shop(&self) -> &BuilderShopContainer {
        self.builder_shop
            .as_ref()
            .expect("BuilderShopContainer must be instantiated before Then assertions")
    }

    pub fn layered_shop(&self) -> &LayeredShopContainer {
        self.layered_shop
            .as_ref()
            .expect("LayeredShopContainer must be instantiated before Then assertions")
    }
}
