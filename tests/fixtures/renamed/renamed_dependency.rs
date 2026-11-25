//! A crate that depends on snapfit only under the name di: a dispatched
//! trait and its supertrait, a container with a fallible field, a factory and
//! fields of the dispatched type, all written with `crate = di`.

use std::sync::Arc;

#[di::static_dispatch(crate = di)]
pub trait Named {
    fn name(&self) -> String;
}

#[di::static_dispatch(crate = di)]
pub trait Ledger: Named {
    fn balance(&self) -> u32;
}

#[derive(Default, Clone)]
pub struct Postgres;

impl Named for Postgres {
    fn name(&self) -> String {
        "postgres".to_owned()
    }
}

impl Ledger for Postgres {
    fn balance(&self) -> u32 {
        7
    }
}

#[derive(Default, Clone)]
pub struct Memory;

impl Named for Memory {
    fn name(&self) -> String {
        "memory".to_owned()
    }
}

impl Ledger for Memory {
    fn balance(&self) -> u32 {
        0
    }
}

/// A loaded setting, whose constructor can fail.
#[derive(Clone)]
pub struct Loaded;

impl Loaded {
    /// Loads the setting.
    pub fn load() -> Result<Self, std::io::Error> {
        Ok(Self)
    }
}

/// A container whose fields reach snapfit through di.
#[di::container(crate = di)]
#[static_dispatch_for(Ledger as Ledgers)]
#[static_dispatch_for(Named as Names)]
#[cfg_attr(all(), extend_static_dispatch(Ledgers, [Memory]))]
pub struct Shop {
    #[singleton(fallible = true, build = constructor(load))]
    loaded: Loaded,

    #[singleton(implementation = Postgres, build = default)]
    ledger: Arc<Ledgers>,

    #[singleton(implementation = Postgres, build = default)]
    name: Names,

    #[singleton(deps = [loaded], build = factory)]
    chosen: Ledgers,
}

impl ShopFactories for Shop {
    fn build_chosen(inputs: ShopChosenInputs) -> Ledgers {
        let ShopChosenInputs { loaded: Loaded } = inputs;
        Ledgers::of(Memory)
    }
}

/// The balance and the name the shop's fields give.
pub fn report() -> Result<(u32, String), di::BuildError> {
    let shop = Shop::try_new()?;
    let balance = shop.get_ledger().balance() + shop.get_chosen().balance();
    Ok((balance, shop.get_name().name()))
}
