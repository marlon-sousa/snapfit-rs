use std::sync::Arc;
use snapfit::container;

#[snapfit::static_dispatch]
pub trait Database {
    fn source(&self) -> &'static str;
}

pub trait Store {
    fn shelf(&self) -> &'static str;
}

pub struct MySql;
impl MySql { pub fn new() -> Self { Self } }
impl Database for MySql { fn source(&self) -> &'static str { "mysql" } }
impl Store for MySql { fn shelf(&self) -> &'static str { "mysql" } }

#[container]
#[static_dispatch_for(Database as Db)]
#[static_dispatch_for(Store as Shelf)]
pub struct Shop {
    #[singleton(implementation = MySql)]
    db: Arc<Db>,
    #[singleton(implementation = MySql)]
    shelf: Arc<Shelf>,
}
