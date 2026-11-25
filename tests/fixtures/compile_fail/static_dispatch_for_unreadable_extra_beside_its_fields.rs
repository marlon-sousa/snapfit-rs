use std::sync::Arc;
use snapfit::container;

#[snapfit::static_dispatch]
pub trait Database {
    fn source(&self) -> &'static str;
}

pub struct MySql;
impl MySql { pub fn new() -> Self { Self } }
impl Database for MySql { fn source(&self) -> &'static str { "mysql" } }

#[container]
#[static_dispatch_for(Database as Db, extra = MySql)]
pub struct Shop {
    #[singleton(implementation = MySql)]
    db: Arc<Db>,
}

pub fn source(db: &Db) -> &'static str {
    db.source()
}

pub fn wrapped() -> Db {
    Db::of(MySql)
}
