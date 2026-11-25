//! What the scenarios about dispatching a trait statically hold, kept apart
//! from the rest of the world.

use cucumber::Parameter;
use strum::{EnumString, IntoStaticStr};

use crate::dispatched_containers::accounts::Accounts;
use crate::dispatched_containers::chosen::ChosenAccounts;
use crate::dispatched_containers::conversions::Conversions;
use crate::dispatched_traits::databases::{Db, Mock, MySql, Postgres};
use crate::dispatched_traits::stores::{Counting, Memory, Shelves};
use crate::dispatched_traits::written_by_a_macro::Racks;

/// A database on the list, as a scenario names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr, Parameter)]
#[strum(serialize_all = "lowercase")]
#[param(name = "database", regex = "mysql|postgres|mock")]
pub enum DatabaseKind {
    Mysql,
    Postgres,
    Mock,
}

impl DatabaseKind {
    pub fn dispatched(self) -> Db {
        match self {
            Self::Mysql => Db::of(MySql::default()),
            Self::Postgres => Db::of(Postgres::default()),
            Self::Mock => Db::of(Mock::default()),
        }
    }
}

/// A store on the list, as a scenario names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Parameter)]
#[strum(serialize_all = "lowercase")]
#[param(name = "store", regex = "memory|counting")]
pub enum StoreKind {
    Memory,
    Counting,
}

impl StoreKind {
    pub fn dispatched(self) -> Shelves<'static> {
        match self {
            Self::Memory => Shelves::of(Memory::default()),
            Self::Counting => Shelves::of(Counting::default()),
        }
    }
}

#[derive(Default)]
pub struct DispatchRuns {
    pub database: Option<Db>,
    pub shelves: Option<Shelves<'static>>,
    /// What the last put of an item answered.
    pub put: Option<usize>,
    pub rack: Option<Racks>,
    pub accounts: Option<Accounts>,
    pub conversions: Option<Conversions>,
    pub chosen: Option<ChosenAccounts>,
    /// What a dispatched lamp answered when it shone.
    pub shone: Option<&'static str>,
}

impl DispatchRuns {
    pub fn database(&self) -> &Db {
        self.database
            .as_ref()
            .expect("a dispatched database must be made in a Given step")
    }

    pub fn rack(&self) -> &Racks {
        self.rack
            .as_ref()
            .expect("a dispatched rack must be made in a Given step")
    }

    pub fn accounts(&self) -> &Accounts {
        self.accounts
            .as_ref()
            .expect("the Accounts container must be built in a Given step")
    }

    pub fn conversions(&self) -> &Conversions {
        self.conversions
            .as_ref()
            .expect("the Conversions container must be built in a Given step")
    }

    pub fn chosen(&self) -> &ChosenAccounts {
        self.chosen
            .as_ref()
            .expect("the ChosenAccounts container must be built in a Given step")
    }

    pub fn database_mut(&mut self) -> &mut Db {
        self.database
            .as_mut()
            .expect("a dispatched database must be made in a Given step")
    }
}
