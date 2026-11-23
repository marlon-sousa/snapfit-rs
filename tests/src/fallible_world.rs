//! What the scenarios about fallible fields built, kept apart from the rest of the world.

use snapfit::{BuildError, FallibleContainer};

use crate::fallible_containers::till::written_by_a_macro::{WholeTill, WordTill};
use crate::fallible_containers::{opening, outage, till};

#[derive(Default)]
pub struct FallibleRuns {
    shop: Option<opening::Shop>,
    till: Option<till::Till>,
    failure: Option<BuildError>,
    events: Vec<&'static str>,
    passed: Vec<(&'static str, u32)>,
}

/// Builds any container that can fail, through the trait it implements.
fn try_build<C: FallibleContainer>() -> Result<C, BuildError> {
    C::try_new()
}

impl FallibleRuns {
    pub fn open_shop(&mut self) {
        self.shop = Some(
            try_build::<opening::Shop>()
                .expect("every field of the opening shop builds without an error"),
        );
    }

    pub fn shop(&self) -> &opening::Shop {
        self.shop
            .as_ref()
            .expect("the shop must be opened before Then assertions")
    }

    pub fn open_till(&mut self) {
        self.till =
            Some(try_build::<till::Till>().expect("the till's ledger builds without an error"));
    }

    pub fn till(&self) -> &till::Till {
        self.till
            .as_ref()
            .expect("the till must be opened before Then assertions")
    }

    pub fn open_storefront(&mut self) {
        outage::take_events();
        self.failure = outage::Storefront::try_new().err();
        self.events = outage::take_events();
    }

    pub fn failure(&self) -> &BuildError {
        self.failure
            .as_ref()
            .expect("the storefront must have failed to open before Then assertions")
    }

    pub fn events(&self) -> &[&'static str] {
        &self.events
    }

    pub fn open_macro_tills(&mut self) {
        let whole = try_build::<WholeTill>().expect("the whole till's ledger builds");
        let word = try_build::<WordTill>().expect("the word till's ledger builds");
        self.passed = vec![
            ("the whole struct", whole.get_ledger().entries),
            ("the fallible word", word.get_ledger().entries),
        ];
    }

    pub fn passed_ledger_entries(&self, form: &str) -> u32 {
        self.passed
            .iter()
            .find(|(passed, _)| *passed == form)
            .map(|(_, entries)| *entries)
            .unwrap_or_else(|| panic!("no till passed {form} was opened"))
    }
}
