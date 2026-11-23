//! The fixture a scenario names, and what compiling or linting it reported.

use compile_outcomes::Outcome;

use super::ContainerWorld;
use crate::fixture_outcomes::{fixture_outcome, lint_outcome};

impl ContainerWorld {
    pub fn select_fixture(&mut self, fixture_name: String) {
        self.selected_fixture = Some(fixture_name);
    }

    pub fn compile_selected_fixture(&mut self) {
        let name = self
            .selected_fixture
            .as_deref()
            .expect("fixture name must be selected in Given step");
        self.compile_outcome = Some(fixture_outcome(name));
    }

    pub fn lint_selected_fixture(&mut self) {
        let name = self
            .selected_fixture
            .as_deref()
            .expect("fixture name must be selected in Given step");
        self.compile_outcome = Some(lint_outcome(name));
    }

    pub fn compiled_fixture(&self) -> (&str, &'static Outcome) {
        let name = self
            .selected_fixture
            .as_deref()
            .expect("fixture name must be selected in Given step");
        let outcome = self
            .compile_outcome
            .expect("fixture must be compiled before Then assertions");
        (name, outcome)
    }
}
