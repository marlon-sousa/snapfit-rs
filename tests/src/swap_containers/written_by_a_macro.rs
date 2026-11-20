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
