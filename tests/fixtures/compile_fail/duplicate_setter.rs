use std::sync::Arc;
use snapfit::container;

#[derive(Default)]
pub struct FolderFiles;

#[derive(Default)]
pub struct MemoryFiles;

pub struct SiteWriter;

#[container(leaf_build = default)]
pub struct Site {
    #[singleton]
    folder_files: Arc<FolderFiles>,

    #[singleton]
    memory_files: Arc<MemoryFiles>,

    #[singleton(deps = [disk = folder_files, disk = memory_files], build = builder)]
    site_writer: Arc<SiteWriter>,
}
