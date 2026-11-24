//! What the regression scenarios about swapping a field's type saw: the
//! desk whose reader is local to its thread, the desks a `macro_rules!`
//! passes their implementation to as tokens, and the desk whose field type it
//! passes.

use std::sync::Arc;

use super::{Source, SwapRuns};
use crate::swap_containers::local::{LocalDesk, LocalReader};
use crate::swap_containers::readers::Reader;
use crate::swap_containers::written_by_a_macro::{IdentDesk, TokenDesk, TypedDesk, WholeDesk};

impl SwapRuns {
    pub fn open_local_desk(&mut self) {
        let reader: Arc<dyn LocalReader> = LocalDesk::new().get_reader();
        self.local = Some(Source::read(reader.source()));
    }

    pub fn local(&self) -> Source {
        self.local
            .expect("the local desk must be opened before Then assertions")
    }

    pub fn open_passed_desks(&mut self) {
        let reads = |reader: Arc<dyn Reader>| Source::read(reader.source());
        self.passed = Some(vec![
            ("a token tree", reads(TokenDesk::new().get_reader())),
            ("an identifier", reads(IdentDesk::new().get_reader())),
            ("a whole struct", reads(WholeDesk::new().get_reader())),
        ]);
    }

    pub fn open_typed_desk(&mut self) {
        self.typed = Some(Source::read(TypedDesk::new().get_reader().source()));
    }

    pub fn typed(&self) -> Source {
        self.typed
            .expect("the typed desk must be opened before Then assertions")
    }

    pub fn passed(&self, form: &str) -> Source {
        self.passed
            .iter()
            .flatten()
            .find(|(passed, _)| *passed == form)
            .map(|(_, source)| *source)
            .unwrap_or_else(|| panic!("no desk passed as {form} was opened"))
    }
}
