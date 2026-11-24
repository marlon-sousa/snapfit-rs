//! What the regression scenarios about swapping a field's type saw: the
//! desk whose reader is local to its thread, the desks a `macro_rules!`
//! passes their implementation to as tokens, and the desk whose field type it
//! passes.

use std::sync::Arc;

use cucumber::Parameter;
use strum::EnumString;

use super::{Source, SwapRuns};
use crate::swap_containers::local::{LocalDesk, LocalReader};
use crate::swap_containers::readers::Reader;
use crate::swap_containers::written_by_a_macro::{IdentDesk, TokenDesk, TypedDesk, WholeDesk};

/// How a `macro_rules!` passes a desk its implementation, as a scenario names
/// it.
#[derive(Debug, Clone, Copy, EnumString, Parameter)]
#[param(
    name = "passed_form",
    regex = "a token tree|an identifier|a whole struct"
)]
pub enum PassedForm {
    #[strum(serialize = "a token tree")]
    TokenTree,
    #[strum(serialize = "an identifier")]
    Identifier,
    #[strum(serialize = "a whole struct")]
    WholeStruct,
}

/// Where each desk whose implementation a `macro_rules!` passes reads from.
#[derive(Debug, Clone, Copy)]
pub struct PassedSources {
    token_tree: Source,
    identifier: Source,
    whole_struct: Source,
}

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
        self.passed = Some(PassedSources {
            token_tree: reads(TokenDesk::new().get_reader()),
            identifier: reads(IdentDesk::new().get_reader()),
            whole_struct: reads(WholeDesk::new().get_reader()),
        });
    }

    pub fn open_typed_desk(&mut self) {
        self.typed = Some(Source::read(TypedDesk::new().get_reader().source()));
    }

    pub fn typed(&self) -> Source {
        self.typed
            .expect("the typed desk must be opened before Then assertions")
    }

    pub fn passed(&self, form: PassedForm) -> Source {
        let passed = self
            .passed
            .expect("the passed desks must be opened before Then assertions");
        match form {
            PassedForm::TokenTree => passed.token_tree,
            PassedForm::Identifier => passed.identifier,
            PassedForm::WholeStruct => passed.whole_struct,
        }
    }
}
