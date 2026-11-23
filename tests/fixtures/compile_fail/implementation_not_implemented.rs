use std::sync::Arc;
use snapfit::container;

pub trait Reader {
    fn source(&self) -> &'static str;
}

pub struct DiskReader;

impl DiskReader {
    pub fn new() -> Self {
        Self
    }
}

impl Reader for DiskReader {
    fn source(&self) -> &'static str {
        "disk"
    }
}

pub struct Notebook;

impl Notebook {
    pub fn new() -> Self {
        Self
    }
}

#[container]
pub struct Desk {
    #[singleton(implementation = Notebook)]
    reader: Arc<dyn Reader>,
}
