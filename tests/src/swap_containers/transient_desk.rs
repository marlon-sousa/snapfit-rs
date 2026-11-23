//! A desk whose reader is a transient trait object, built afresh from the
//! implementation it names for every service and every get.

use std::sync::Arc;

use snapfit::container;

use super::readers::{DiskReader, Reader, Report};

#[container]
pub struct TransientDesk {
    #[transient(implementation = DiskReader)]
    reader: Arc<dyn Reader>,

    #[singleton(deps = [reader])]
    report: Arc<Report>,
}
