use std::sync::Arc;
use snapfit::container;

#[snapfit::static_dispatch]
pub trait Database {
    fn source(&self) -> &'static str;
}

#[derive(Clone)]
pub struct MySql;
impl Database for MySql { fn source(&self) -> &'static str { "mysql" } }

#[derive(Clone)]
pub struct Postgres;
impl Database for Postgres { fn source(&self) -> &'static str { "postgres" } }

#[container]
#[static_dispatch_for(Database as Db)]
pub struct Shop {
    #[singleton(implementation = MySql)]
    db: Db,

    #[singleton(implementation = Postgres)]
    replica: Arc<Db>,

    #[singleton(implementation = MySql)]
    backup: Arc<Db>,
}
