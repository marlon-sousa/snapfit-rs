//! The containers whose fields say how they are built, kept apart from the rest of the world.

use crate::build_containers::{archive, generated, leaves, named};
use crate::builder_containers::{catalog, handmade, site};
use crate::dispatch_world::DispatchRuns;
use crate::factory_containers::{orders, payments};
use crate::fallible_world::FallibleRuns;
use crate::swap_world::SwapRuns;

#[derive(Default)]
pub struct BuildContainers {
    books: Option<named::Books>,
    billing: Option<leaves::Billing>,
    archive: Option<archive::Archive>,
    shop: Option<orders::Shop>,
    store: Option<payments::Store>,
    site: Option<site::Site>,
    shelves: Option<catalog::Shelves>,
    reports: Option<handmade::Reports>,
    office: Option<generated::Office>,
    front_desk: Option<generated::FrontDesk>,
    pub dispatch: DispatchRuns,
    pub fallible: FallibleRuns,
    pub swaps: SwapRuns,
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

    pub fn instantiate_site(&mut self) {
        self.site = Some(site::Site::new());
    }

    pub fn site(&self) -> &site::Site {
        self.site
            .as_ref()
            .expect("the Site container must be instantiated before Then assertions")
    }

    pub fn instantiate_shelves(&mut self) {
        self.shelves = Some(catalog::Shelves::new());
    }

    pub fn shelves(&self) -> &catalog::Shelves {
        self.shelves
            .as_ref()
            .expect("the Shelves container must be instantiated before Then assertions")
    }

    pub fn instantiate_reports(&mut self) {
        self.reports = Some(handmade::Reports::new());
    }

    pub fn reports(&self) -> &handmade::Reports {
        self.reports
            .as_ref()
            .expect("the Reports container must be instantiated before Then assertions")
    }

    pub fn instantiate_office(&mut self) {
        self.office = Some(generated::Office::new());
    }

    pub fn office(&self) -> &generated::Office {
        self.office
            .as_ref()
            .expect("the Office container must be instantiated before Then assertions")
    }

    pub fn instantiate_front_desk(&mut self) {
        self.front_desk = Some(generated::FrontDesk::new());
    }

    pub fn front_desk(&self) -> &generated::FrontDesk {
        self.front_desk
            .as_ref()
            .expect("the FrontDesk container must be instantiated before Then assertions")
    }
}
