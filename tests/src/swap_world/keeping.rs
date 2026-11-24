//! What the scenario about services that keep the reader's trait object
//! inside their own type saw.

use cucumber::Parameter;
use strum::EnumString;

use super::{Source, SwapRuns};
use crate::swap_containers::keeping::{KeepingDesk, TypedKeepingDesk};

/// How a service that keeps the reader's trait object is written and built,
/// as a scenario names it.
#[derive(Debug, Clone, Copy, EnumString, Parameter)]
#[param(
    name = "kept_form",
    regex = "an Arc built by new|a value built by a constructor|an Arc a macro_rules! passes as a type"
)]
pub enum KeptForm {
    #[strum(serialize = "an Arc built by new")]
    ArcBuiltByNew,
    #[strum(serialize = "a value built by a constructor")]
    ValueBuiltByConstructor,
    #[strum(serialize = "an Arc a macro_rules! passes as a type")]
    ArcPassedAsType,
}

/// Where each service that keeps the reader's trait object reads from.
#[derive(Debug, Clone, Copy)]
pub struct KeptSources {
    arc_built_by_new: Source,
    value_built_by_constructor: Source,
    arc_passed_as_type: Source,
}

impl SwapRuns {
    pub fn open_keeping_desks(&mut self) {
        let desk = KeepingDesk::new();
        self.kept = Some(KeptSources {
            arc_built_by_new: Source::read(desk.get_shelf().reader.source()),
            value_built_by_constructor: Source::read(desk.get_stocked().reader.source()),
            arc_passed_as_type: Source::read(TypedKeepingDesk::new().get_shelf().reader.source()),
        });
    }

    pub fn kept(&self, form: KeptForm) -> Source {
        let kept = self
            .kept
            .expect("the keeping desks must be opened before Then assertions");
        match form {
            KeptForm::ArcBuiltByNew => kept.arc_built_by_new,
            KeptForm::ValueBuiltByConstructor => kept.value_built_by_constructor,
            KeptForm::ArcPassedAsType => kept.arc_passed_as_type,
        }
    }
}
