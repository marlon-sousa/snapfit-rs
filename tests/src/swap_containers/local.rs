//! A desk whose trait, and the implementation it names, are not `Send` and
//! not `Sync`, so that clippy's `arc_with_non_send_sync` would fire on an
//! `Arc::new` the macro writes if it were linted as the user's own code.

use std::rc::Rc;
use std::sync::Arc;

use snapfit::container;

pub trait LocalReader {
    fn source(&self) -> &str;
}

pub struct RcReader {
    source: Rc<str>,
}

impl RcReader {
    pub fn new() -> Self {
        Self {
            source: Rc::from("disk"),
        }
    }
}

impl LocalReader for RcReader {
    fn source(&self) -> &str {
        &self.source
    }
}

#[container]
pub struct LocalDesk {
    #[singleton(implementation = RcReader)]
    reader: Arc<dyn LocalReader>,
}
