use std::sync::Arc;
use snapfit::container;

#[snapfit::static_dispatch]
pub trait Database {
    fn source(&self) -> &'static str;
}

pub struct MySql;
impl MySql { pub fn new() -> Self { Self } }
impl Database for MySql { fn source(&self) -> &'static str { "mysql" } }

pub struct Postgres;
impl Database for Postgres { fn source(&self) -> &'static str { "postgres" } }

#[container]
#[static_dispatch_for(Database as Db)]
pub struct Shop {
    #[singleton(implementation = MySql)]
    db: Arc<Db>,

    #[singleton(implementations = [MySql, Postgres])]
    replica: Arc<Db>,
}
