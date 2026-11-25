//! A public container whose dispatched type holds an implementation more
//! private than the container, and one that is not `Send` and not `Sync`,
//! held in an `Arc` and as a value, for which neither rustc nor clippy may
//! warn about the alias, the getters and the factory inputs snapfit writes.
//! Code outside the module could not use a value of the alias, which holds
//! a type private to the module.

use std::rc::Rc;
use std::sync::Arc;
use snapfit::container;

pub mod shop {
    use super::*;

    #[snapfit::static_dispatch]
    pub trait Reader {
        fn source(&self) -> String;
    }

    #[derive(Default, Clone)]
    pub struct RcReader {
        source: Rc<str>,
    }

    impl Reader for RcReader {
        fn source(&self) -> String {
            self.source.to_string()
        }
    }

    #[derive(Default, Clone)]
    struct Mock;

    impl Reader for Mock {
        fn source(&self) -> String {
            "mock".to_owned()
        }
    }

    pub struct Summary;

    #[container]
    #[static_dispatch_for(Reader as Readers)]
    #[cfg_attr(all(), extend_static_dispatch(Readers, [Mock]))]
    pub struct Desk {
        #[singleton(implementation = RcReader, build = default)]
        reader: Arc<Readers>,

        #[singleton(implementation = Mock, build = default)]
        copy: Readers,

        #[singleton(deps = [reader], build = factory)]
        summary: Arc<Summary>,
    }

    impl DeskFactories for Desk {
        fn build_summary(inputs: DeskSummaryInputs) -> Arc<Summary> {
            let _ = inputs.reader.source();
            Arc::new(Summary)
        }
    }

    /// What the container's reader, its copy and a mock built by hand read.
    pub fn read() -> (String, String, String) {
        let desk = Desk::new();
        (desk.get_reader().source(), desk.get_copy().source(), Readers::of(Mock).source())
    }
}
