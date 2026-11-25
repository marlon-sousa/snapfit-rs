//! A container whose fields hold one dispatched type, each built from the
//! implementation it names, beside a field that dispatches the same trait
//! dynamically. An extension whose condition holds adds `Spare`, and one
//! whose condition is false names a type that does not exist.

use std::sync::Arc;

use snapfit::container;

use super::ledgers::{Books, Config, Disk, Fake, Ledger, Memory, Remote};

#[container]
#[static_dispatch_for(Ledger as Ledgers, extra = [Fake])]
#[cfg_attr(all(), extend_static_dispatch(Ledgers, [Spare]))]
#[cfg_attr(any(), extend_static_dispatch(Ledgers, [Missing]))]
pub struct Accounts {
    #[singleton]
    config: Arc<Config>,

    #[singleton(implementation = Disk)]
    main: Arc<Ledgers>,

    #[singleton(implementation = Remote, deps = [config], build = constructor(connect))]
    backup: Arc<Ledgers>,

    #[singleton(implementation = Memory, build = default)]
    cache: Arc<Ledgers>,

    #[singleton(implementation = Disk)]
    dynamic: Arc<dyn Ledger>,

    #[singleton(deps = [main])]
    books: Arc<Books>,
}

/// Added by an extension alone, which only a test builds.
pub struct Spare;

impl Ledger for Spare {
    fn source(&self) -> String {
        "spare".to_owned()
    }
}
