//! A public container whose dispatched type holds an implementation more
//! private than the container, and one that is not `Send` and not `Sync`,
//! held in an `Arc` and as a value, built from the implementation they name
//! and by a factory, for which neither rustc nor clippy may warn about the
//! alias, the getters, the factory methods and the factory inputs snapfit
//! writes.
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

        #[singleton(build = factory)]
        chosen: Arc<Readers>,

        #[singleton(build = factory)]
        chosen_copy: Readers,

        #[singleton(deps = [reader], build = factory)]
        summary: Arc<Summary>,
    }

    // The factories are the user's own items, so they name the private
    // implementation themselves and allow the lint, as SPEC, §7, says.
    #[allow(private_interfaces)]
    impl DeskFactories for Desk {
        // The user's own Arc of a type that is not Send, as the field asks.
        #[allow(clippy::arc_with_non_send_sync)]
        fn build_chosen(_inputs: DeskChosenInputs) -> Arc<Readers> {
            Arc::new(Readers::of(Mock))
        }

        fn build_chosen_copy(_inputs: DeskChosenCopyInputs) -> Readers {
            Readers::of(RcReader::default())
        }

        fn build_summary(inputs: DeskSummaryInputs) -> Arc<Summary> {
            let _ = inputs.reader.source();
            Arc::new(Summary)
        }
    }

    /// What the container's reader, its copy, its factories' choices and a
    /// mock built by hand read.
    pub fn read() -> [String; 5] {
        let desk = Desk::new();
        [
            desk.get_reader().source(),
            desk.get_copy().source(),
            desk.get_chosen().source(),
            desk.get_chosen_copy().source(),
            Readers::of(Mock).source(),
        ]
    }
}
