use std::sync::Arc;
use snapfit::container;

#[derive(Default)]
pub struct FolderFiles;

pub struct SiteReader;

impl SiteReader {
    pub fn new(_disk: Arc<FolderFiles>) -> Self {
        Self
    }
}

#[container(leaf_build = default)]
pub struct Site {
    #[singleton]
    folder_files: Arc<FolderFiles>,

    #[singleton(deps = [disk = folder_files])]
    site_reader: Arc<SiteReader>,
}
