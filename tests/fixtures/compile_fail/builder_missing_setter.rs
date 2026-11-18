use std::sync::Arc;
use snapfit::container;

#[derive(Default)]
pub struct FolderFiles;

pub struct SiteWriter;

#[bon::bon]
impl SiteWriter {
    #[builder(start_fn = builder)]
    pub fn new(_disk: Arc<FolderFiles>, _memory: Arc<FolderFiles>) -> Self {
        Self
    }
}

#[container(leaf_build = default)]
pub struct Site {
    #[singleton]
    folder_files: Arc<FolderFiles>,

    #[singleton(deps = [disk = folder_files], build = builder)]
    site_writer: Arc<SiteWriter>,
}
