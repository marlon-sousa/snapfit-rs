use std::sync::Arc;
use snapfit::container;

#[snapfit::static_dispatch]
pub trait Database {
    type Row;

    fn find(&self) -> Self::Row;
}

pub struct MySql;
impl MySql { pub fn new() -> Self { Self } }
impl Database for MySql { type Row = u32; fn find(&self) -> u32 { 1 } }

pub struct Postgres;
impl Postgres { pub fn new() -> Self { Self } }
impl Database for Postgres { type Row = String; fn find(&self) -> String { "1".to_owned() } }

#[container]
#[static_dispatch_for(Database as Db)]
pub struct Shop {
    #[singleton(implementation = MySql)]
    orders_db: Arc<Db>,

    #[singleton(implementation = Postgres)]
    reports_db: Arc<Db>,
}
