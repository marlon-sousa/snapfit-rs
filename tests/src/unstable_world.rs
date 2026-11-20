use std::error::Error;

use crate::unstable_build_containers::{DepotContainer, KioskContainer};
use crate::unstable_containers::{
    AsyncShopContainer, LayeredShopContainer, TransientFactoryShopContainer, TransientShopContainer,
};

#[derive(Default)]
pub struct UnstableContainers {
    async_shop: Option<AsyncShopContainer>,
    transient_shop: Option<TransientShopContainer>,
    transient_factory_shop: Option<TransientFactoryShopContainer>,
    layered_shop: Option<LayeredShopContainer>,
    depot: Option<DepotContainer>,
    kiosk_failure: Option<Box<dyn Error + Send + Sync>>,
}

impl UnstableContainers {
    pub async fn init_async_shop(&mut self) {
        let container = AsyncShopContainer::init()
            .await
            .unwrap_or_else(|err| panic!("AsyncShopContainer::init() failed: {err}"));
        self.async_shop = Some(container);
    }

    pub async fn init_depot(&mut self) {
        let container = DepotContainer::init()
            .await
            .unwrap_or_else(|err| panic!("DepotContainer::init() failed: {err}"));
        self.depot = Some(container);
    }

    pub async fn init_kiosk(&mut self) {
        self.kiosk_failure = KioskContainer::init().await.err();
    }

    pub fn kiosk_failure(&self) -> &(dyn Error + Send + Sync + 'static) {
        self.kiosk_failure
            .as_deref()
            .expect("KioskContainer must have failed to initialize before Then assertions")
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

    pub fn depot(&self) -> &DepotContainer {
        self.depot
            .as_ref()
            .expect("DepotContainer must be initialized before Then assertions")
    }
}
