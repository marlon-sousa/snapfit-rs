use std::sync::Arc;
use snapfit::container;

#[snapfit::static_dispatch]
pub trait Database {
    fn source(&self) -> &'static str;
}

pub struct MySql;
impl MySql { pub fn new() -> Self { Self } }
impl Database for MySql { fn source(&self) -> &'static str { "mysql" } }

pub struct Csv;
impl Csv { pub fn new() -> Self { Self } }

#[container]
#[static_dispatch_for(Database as Db)]
pub struct Shop {
    #[singleton(build = factory, implementations = [MySql, Csv])]
    db: Arc<Db>,
}

impl ShopFactories for Shop {
    fn build_db(_inputs: ShopDbInputs) -> Arc<Db> {
        Arc::new(Db::of(MySql))
    }
}
