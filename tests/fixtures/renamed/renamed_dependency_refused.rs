//! A container of a crate that depends on snapfit only under the name di,
//! refused for a mode snapfit does not have, whose dispatched type snapfit
//! still writes through di.

#[di::static_dispatch(crate = di)]
pub trait Ledger {
    fn balance(&self) -> u32;
}

#[derive(Default, Clone)]
pub struct Postgres;

impl Ledger for Postgres {
    fn balance(&self) -> u32 {
        7
    }
}

#[di::container(mode = "threaded", crate = di)]
#[static_dispatch_for(Ledger as Ledgers)]
pub struct Shop {
    #[singleton(implementation = Postgres, build = default)]
    ledger: Ledgers,
}

/// The dispatched type, which code outside the container still names.
pub fn balance() -> u32 {
    Ledgers::of(Postgres).balance()
}
