//! Two desks whose reader is the trait object, and which differ only in the
//! implementation each names, so that every service, its factory included,
//! is written the same way in both.

use std::sync::Arc;

use snapfit::container;

use super::readers::{Digest, Report, Summary};
use super::readers::{DiskReader, MemoryReader, Reader};

#[container]
pub struct DiskDesk {
    #[singleton(implementation = DiskReader)]
    reader: Arc<dyn Reader>,

    #[singleton(deps = [reader])]
    report: Arc<Report>,

    #[singleton(deps = [reader], build = builder)]
    digest: Arc<Digest>,

    #[singleton(deps = [reader], build = factory)]
    summary: Arc<Summary>,
}

impl DiskDeskFactories for DiskDesk {
    fn build_summary(inputs: DiskDeskSummaryInputs) -> Arc<Summary> {
        Arc::new(Summary::from_reader(inputs.reader))
    }
}

#[container]
pub struct MemoryDesk {
    #[singleton(implementation = MemoryReader)]
    reader: Arc<dyn Reader>,

    #[singleton(deps = [reader])]
    report: Arc<Report>,

    #[singleton(deps = [reader], build = builder)]
    digest: Arc<Digest>,

    #[singleton(deps = [reader], build = factory)]
    summary: Arc<Summary>,
}

impl MemoryDeskFactories for MemoryDesk {
    fn build_summary(inputs: MemoryDeskSummaryInputs) -> Arc<Summary> {
        Arc::new(Summary::from_reader(inputs.reader))
    }
}
