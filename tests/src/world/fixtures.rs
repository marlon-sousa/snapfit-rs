//! The fixture a scenario names, and what compiling or linting it reported.

use compile_outcomes::Outcome;

use super::ContainerWorld;
use crate::fixture_outcomes::{fixture_outcome, lint_outcome};

impl ContainerWorld {
    pub fn select_fixture(&mut self, fixture_name: String) {
        self.selected_fixture = Some(fixture_name);
    }

    pub fn compile_selected_fixture(&mut self) {
        self.compile_outcome = Some(fixture_outcome(self.selected_fixture()));
    }

    pub fn lint_selected_fixture(&mut self) {
        self.compile_outcome = Some(lint_outcome(self.selected_fixture()));
    }

    pub fn compiled_fixture(&self) -> (&str, &'static Outcome) {
        let outcome = self
            .compile_outcome
            .expect("fixture must be compiled before Then assertions");
        (self.selected_fixture(), outcome)
    }

    fn selected_fixture(&self) -> &str {
        self.selected_fixture
            .as_deref()
            .expect("fixture name must be selected in Given step")
    }
}
