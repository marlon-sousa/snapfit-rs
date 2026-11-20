//! A container whose `leaf_build` is written `constructor(new)`, which
//! builds a fallible leaf the way the leaf's own `new` already would.

use std::sync::Arc;

use snapfit::container;

pub struct Ledger {
    pub entries: u32,
}

impl Ledger {
    pub fn new() -> Result<Self, String> {
        Ok(Self { entries: 0 })
    }
}

#[container(leaf_build = constructor(new))]
pub struct Till {
    #[singleton(fallible = true)]
    ledger: Arc<Ledger>,
}
