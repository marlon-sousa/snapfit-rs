//! A container whose dispatched field fails to build, and returns its error
//! from `try_new()`.

use std::sync::Arc;

use snapfit::container;

use super::ledgers::{Config, Ledger, Remote};

#[container]
#[static_dispatch_for(Ledger as Checked)]
pub struct CheckedAccounts {
    #[singleton(build = constructor(offline))]
    config: Arc<Config>,

    #[singleton(implementation = Remote, deps = [config], build = constructor(try_connect), fallible = true)]
    remote: Arc<Checked>,
}
