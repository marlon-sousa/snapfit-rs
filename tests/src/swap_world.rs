//! What the scenarios about swapping a field's type saw, kept apart from the rest of the world.

use std::sync::Arc;

use cucumber::Parameter;
use snapfit::BuildError;
use strum::EnumString;

use crate::swap_containers::built_desks::{BuiltDesk, LeafDesk, LockedDesk};
use crate::swap_containers::desks::{DiskDesk, MemoryDesk};
use crate::swap_containers::readers::Reader;
use crate::swap_containers::written_by_a_macro::MacroDesk;

/// The reader a container builds, as a scenario names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Parameter)]
#[strum(serialize_all = "lowercase")]
#[param(name = "source", regex = "disk|memory")]
pub enum Source {
    Disk,
    Memory,
}

impl Source {
    fn read(name: &str) -> Self {
        name.parse()
            .unwrap_or_else(|_| panic!("no reader reads from {name}"))
    }
}

/// Where each value of an opened desk reads from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeskSources {
    pub reader: Source,
    pub report: Source,
    pub digest: Source,
    pub summary: Source,
}

/// Reads where a desk's values read from; a macro, so that one body serves
/// every desk, which are different types with getters of the same types.
macro_rules! desk_sources {
    ($desk:expr) => {{
        let desk = $desk;
        let reader: Arc<dyn Reader> = desk.get_reader();
        DeskSources {
            reader: Source::read(reader.source()),
            report: Source::read(desk.get_report().reader.source()),
            digest: Source::read(desk.get_digest().reader.source()),
            summary: Source::read(desk.get_summary().reader.source()),
        }
    }};
}

/// What a desk with a transient reader gave: whether two gets of the reader
/// were the same value, and where they and the report read from.
#[cfg(feature = "unstable")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransientReads {
    pub same_reader: bool,
    pub reader: Source,
    pub report: Source,
}

/// What each reader of the built desks reads from, by the way it was built,
/// and the error of the desk whose reader cannot be built.
#[derive(Debug)]
pub struct BuiltReads {
    pub sources: Vec<(&'static str, &'static str)>,
    pub locked: BuildError,
}

#[derive(Default)]
pub struct SwapRuns {
    desk: Option<DeskSources>,
    built: Option<BuiltReads>,
    #[cfg(feature = "unstable")]
    transient: Option<TransientReads>,
}

impl SwapRuns {
    pub fn open_desk(&mut self, source: Source) {
        self.desk = Some(match source {
            Source::Disk => desk_sources!(DiskDesk::new()),
            Source::Memory => desk_sources!(MemoryDesk::new()),
        });
    }

    pub fn open_macro_desk(&mut self) {
        self.desk = Some(desk_sources!(MacroDesk::new()));
    }

    pub fn desk(&self) -> DeskSources {
        self.desk
            .expect("the desk must be opened before Then assertions")
    }

    #[cfg(feature = "unstable")]
    pub fn open_transient_desk(&mut self) {
        let desk = crate::swap_containers::transient_desk::TransientDesk::new();
        let (first, second): (Arc<dyn Reader>, Arc<dyn Reader>) =
            (desk.get_reader(), desk.get_reader());
        self.transient = Some(TransientReads {
            same_reader: Arc::ptr_eq(&first, &second),
            reader: Source::read(first.source()),
            report: Source::read(desk.get_report().reader.source()),
        });
    }

    #[cfg(feature = "unstable")]
    pub fn transient(&self) -> TransientReads {
        self.transient
            .expect("the transient desk must be opened before Then assertions")
    }

    pub fn open_built_desks(&mut self) {
        let desk = BuiltDesk::try_new()
            .unwrap_or_else(|error| panic!("the built desk did not open: {error}"));
        let leaf = LeafDesk::new();
        let locked = LockedDesk::try_new()
            .err()
            .expect("the locked desk must fail to open");
        self.built = Some(BuiltReads {
            sources: vec![
                ("opened", desk.get_opened().source()),
                ("defaulted", desk.get_defaulted().source()),
                ("built", desk.get_built().source()),
                ("checked", desk.get_checked().source()),
                ("leaf", leaf.get_reader().source()),
            ],
            locked,
        });
    }

    pub fn built(&self) -> &BuiltReads {
        self.built
            .as_ref()
            .expect("the built desks must be opened before Then assertions")
    }
}
