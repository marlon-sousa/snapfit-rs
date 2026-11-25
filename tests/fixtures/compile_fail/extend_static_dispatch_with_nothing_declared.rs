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
#[extend_static_dispatch(Db, [MySql])]
pub struct Shop {
    #[singleton]
    db: Arc<MySql>,
}
