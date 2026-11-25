//! snapfit, reached as `di` by the fixtures that depend on this crate alone,
//! and, in `renamed`, a dispatched trait and a container of a crate that
//! depends on snapfit under the name `inject`.

pub use inject::*;

/// Code that builds only if `crate = inject` reaches everything snapfit writes.
pub mod renamed {
    #[inject::static_dispatch(crate = inject)]
    pub trait Ledger {
        fn balance(&self) -> u32;
    }

    /// A ledger kept on disk.
    #[derive(Default, Clone)]
    pub struct Disk;

    impl Ledger for Disk {
        fn balance(&self) -> u32 {
            7
        }
    }

    /// A container whose ledger is dispatched statically.
    #[inject::container(crate = inject)]
    #[static_dispatch_for(Ledger as Ledgers)]
    pub struct Books {
        #[singleton(implementation = Disk, build = default)]
        ledger: Ledgers,
    }

    /// The balance of the books' ledger.
    pub fn balance() -> u32 {
        Books::new().get_ledger().balance()
    }
}
