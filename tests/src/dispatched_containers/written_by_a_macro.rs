//! A container written by a `macro_rules!`, which passes the dispatched
//! field's type, its implementation and the `extra` list's type as types.

use std::sync::Arc;

use snapfit::container;

use super::ledgers::{Disk, Fake, Ledger};

macro_rules! accounts {
    ($container:ident, $field:ty, $implementation:ty, $extra:ty) => {
        #[container]
        #[static_dispatch_for(Ledger as Passed, extra = [$extra])]
        pub struct $container {
            #[singleton(implementation = $implementation)]
            main: $field,
        }
    };
}

accounts!(PassedAccounts, Arc<Passed>, Disk, Fake);
