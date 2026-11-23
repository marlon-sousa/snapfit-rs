//! A desk whose implementation, written by hand and passed by a `macro_rules!`,
//! is not `Send` and not `Sync`, for which clippy must not lint the `Arc::new`
//! snapfit writes.

use std::rc::Rc;
use std::sync::Arc;
use snapfit::container;

pub trait Reader {
    fn source(&self) -> &str;
}

#[derive(Default)]
pub struct RcReader {
    source: Rc<str>,
}

impl Reader for RcReader {
    fn source(&self) -> &str {
        &self.source
    }
}

#[container]
pub struct Desk {
    #[singleton(implementation = RcReader, build = default)]
    reader: Arc<dyn Reader>,
}

macro_rules! token_desk {
    ($desk:ident, $implementation:tt) => {
        #[container]
        pub struct $desk {
            #[singleton(implementation = $implementation, build = default)]
            reader: Arc<dyn Reader>,
        }
    };
}

token_desk!(TokenDesk, RcReader);
