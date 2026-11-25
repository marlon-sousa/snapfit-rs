//! A container written by a `macro_rules!`, which passes the dispatched
//! field's type, its implementation and the type a factory field lists as types.

use std::sync::Arc;

use snapfit::container;

use super::ledgers::{Disk, Fake, Ledger};

macro_rules! accounts {
    ($container:ident, $field:ty, $implementation:ty, $listed:ty) => {
        #[container]
        #[static_dispatch_for(Ledger as Passed)]
        pub struct $container {
            #[singleton(implementation = $implementation)]
            main: $field,

            #[singleton(build = factory, implementation = [$listed])]
            chosen: $field,
        }
    };
}

accounts!(PassedAccounts, Arc<Passed>, Disk, Fake);

impl PassedAccountsFactories for PassedAccounts {
    fn build_chosen(_inputs: PassedAccountsChosenInputs) -> Arc<Passed> {
        Arc::new(Passed::of(Fake))
    }
}
