//! The trait both readers implement, the two readers, and the services that
//! depend on a reader, written once for every container that builds them.

use std::sync::Arc;

pub trait Reader: Send + Sync {
    fn source(&self) -> &'static str;
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct MemoryReader;

impl MemoryReader {
    pub fn new() -> Self {
        Self
    }
}

impl Reader for MemoryReader {
    fn source(&self) -> &'static str {
        "memory"
    }
}

/// Built by `new`, taking the trait object.
pub struct Report {
    pub reader: Arc<dyn Reader>,
}

impl Report {
    pub fn new(reader: Arc<dyn Reader>) -> Self {
        Self { reader }
    }
}

/// Built by its builder, whose setter takes anything that converts into the
/// trait object, as `setter(into)` writes it.
#[derive(typed_builder::TypedBuilder)]
pub struct Digest {
    #[builder(setter(into))]
    pub reader: Arc<dyn Reader>,
}

/// Built by a factory, from the reader its inputs struct holds.
pub struct Summary {
    pub reader: Arc<dyn Reader>,
}

impl Summary {
    pub fn from_reader(reader: Arc<dyn Reader>) -> Self {
        Self { reader }
    }
}

/// Generic over the reader, which it takes by value.
pub struct Ledger<R: Reader> {
    pub reader: R,
}

impl<R: Reader> Ledger<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

/// Generic over the reader, which it takes in an `Arc`.
pub struct Journal<R: Reader> {
    pub reader: Arc<R>,
}

impl<R: Reader> Journal<R> {
    pub fn new(reader: Arc<R>) -> Self {
        Self { reader }
    }
}

/// Generic only in `new`, keeping what it read from the reader, so that its
/// own type names no reader and needs no pointer.
pub struct Tally {
    pub source: &'static str,
}

impl Tally {
    pub fn new<R: Reader>(reader: R) -> Self {
        Self {
            source: reader.source(),
        }
    }
}

/// Generic only in `new`, keeping the reader itself, which a type that
/// names no reader can hold only behind a pointer.
pub struct Notebook {
    pub reader: Box<dyn Reader>,
}

impl Notebook {
    pub fn new<R: Reader + 'static>(reader: R) -> Self {
        Self {
            reader: Box::new(reader),
        }
    }
}
