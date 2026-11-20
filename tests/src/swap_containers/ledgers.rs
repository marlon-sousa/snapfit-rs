//! Books whose services are generic over the reader. A service generic on
//! its struct is named with the reader in the container, here through an
//! alias, so that a swap is the alias's line; a service generic only in its
//! `new` names no reader at all.

use std::sync::Arc;

use snapfit::container;

use super::readers::{DiskReader, Journal, Ledger, MemoryReader, Notebook, Tally};

pub type DiskBooksReader = DiskReader;
pub type MemoryBooksReader = MemoryReader;

#[container]
pub struct DiskBooks {
    #[singleton]
    reader: DiskBooksReader,

    #[singleton]
    shared_reader: Arc<DiskBooksReader>,

    #[singleton(deps = [reader])]
    ledger: Arc<Ledger<DiskBooksReader>>,

    #[singleton(deps = [shared_reader])]
    journal: Arc<Journal<DiskBooksReader>>,

    #[singleton(deps = [reader])]
    tally: Arc<Tally>,

    #[singleton(deps = [reader])]
    notebook: Arc<Notebook>,
}

#[container]
pub struct MemoryBooks {
    #[singleton]
    reader: MemoryBooksReader,

    #[singleton]
    shared_reader: Arc<MemoryBooksReader>,

    #[singleton(deps = [reader])]
    ledger: Arc<Ledger<MemoryBooksReader>>,

    #[singleton(deps = [shared_reader])]
    journal: Arc<Journal<MemoryBooksReader>>,

    #[singleton(deps = [reader])]
    tally: Arc<Tally>,

    #[singleton(deps = [reader])]
    notebook: Arc<Notebook>,
}
