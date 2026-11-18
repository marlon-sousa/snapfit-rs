//! The containers whose fields say how they are built, kept apart from the rest of the world.

use crate::build_containers::{archive, leaves, named};
use crate::factory_containers::{orders, payments};

#[derive(Default)]
pub struct BuildContainers {
    books: Option<named::Books>,
    billing: Option<leaves::Billing>,
    archive: Option<archive::Archive>,
    shop: Option<orders::Shop>,
    store: Option<payments::Store>,
}

impl BuildContainers {
    pub fn instantiate_books(&mut self) {
        self.books = Some(named::Books::new());
    }

    pub fn instantiate_archive(&mut self) {
        self.archive = Some(archive::Archive::new());
    }

    pub fn archive(&self) -> &archive::Archive {
        self.archive
            .as_ref()
            .expect("the Archive container must be instantiated before Then assertions")
    }

    pub fn instantiate_billing(&mut self) {
        self.billing = Some(leaves::Billing::new());
    }

    pub fn billing(&self) -> &leaves::Billing {
        self.billing
            .as_ref()
            .expect("the Billing container must be instantiated before Then assertions")
    }

    pub fn books(&self) -> &named::Books {
        self.books
            .as_ref()
            .expect("the Books container must be instantiated before Then assertions")
    }

    pub fn instantiate_shop(&mut self) {
        self.shop = Some(orders::Shop::new());
    }

    pub fn instantiate_store(&mut self) {
        self.store = Some(payments::Store::new());
    }

    pub fn shop(&self) -> &orders::Shop {
        self.shop
            .as_ref()
            .expect("the Shop container must be instantiated before Then assertions")
    }

    pub fn store(&self) -> &payments::Store {
        self.store
            .as_ref()
            .expect("the Store container must be instantiated before Then assertions")
    }
}
