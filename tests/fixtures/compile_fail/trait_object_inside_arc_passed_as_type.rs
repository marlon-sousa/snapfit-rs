use std::sync::Arc;
use snapfit::container;

pub trait Store {}

pub struct Archive<S: ?Sized>(Box<S>);

macro_rules! archive_container {
    ($archive:ty) => {
        #[container]
        pub struct ArchiveContainer {
            #[singleton]
            archive: $archive,
        }
    };
}

archive_container!(Arc<Archive<dyn Store>>);
