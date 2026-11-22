//! A desk written by a `macro_rules!`, which passes the trait object and the
//! implementation as types, so that both and the `deps` entries carry the
//! macro's hygiene.

use std::sync::Arc;

use snapfit::container;

use super::readers::{Digest, DiskReader, Reader, Report, Summary};

macro_rules! desk {
    ($desk:ident, $object:ty, $implementation:ty) => {
        #[container]
        pub struct $desk {
            #[singleton(implementation = $implementation)]
            reader: Arc<$object>,

            #[singleton(deps = [reader])]
            report: Arc<Report>,

            #[singleton(deps = [reader], build = builder)]
            digest: Arc<Digest>,

            #[singleton(deps = [reader], build = factory)]
            summary: Arc<Summary>,
        }
    };
}

desk!(MacroDesk, dyn Reader, DiskReader);

impl MacroDeskFactories for MacroDesk {
    fn build_summary(inputs: MacroDeskSummaryInputs) -> Arc<Summary> {
        Arc::new(Summary::from_reader(inputs.reader))
    }
}

/// A desk whose implementation a `macro_rules!` passes as a token tree.
macro_rules! token_desk {
    ($desk:ident, $implementation:tt) => {
        #[container]
        pub struct $desk {
            #[singleton(implementation = $implementation)]
            reader: Arc<dyn Reader>,
        }
    };
}

/// A desk whose implementation a `macro_rules!` passes as an identifier.
macro_rules! ident_desk {
    ($desk:ident, $implementation:ident) => {
        #[container]
        pub struct $desk {
            #[singleton(implementation = $implementation)]
            reader: Arc<dyn Reader>,
        }
    };
}

/// A container a `macro_rules!` passes whole, as tokens.
macro_rules! whole {
    ($($item:tt)*) => {
        #[container]
        $($item)*
    };
}

token_desk!(TokenDesk, DiskReader);
ident_desk!(IdentDesk, DiskReader);
whole! {
    pub struct WholeDesk {
        #[singleton(implementation = DiskReader)]
        reader: Arc<dyn Reader>,
    }
}
