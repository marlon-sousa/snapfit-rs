use std::sync::Arc;
use snapfit::container;

#[snapfit::static_dispatch]
pub trait Database {
    fn source(&self) -> &'static str;
}

pub struct MySql;
impl MySql { pub fn new() -> Self { Self } }
impl Database for MySql { fn source(&self) -> &'static str { "mysql" } }

macro_rules! shop {
    ($field:ty, $extra:ty) => {
        #[container]
        #[static_dispatch_for(Database as Db, extra = [$extra])]
        pub struct Shop {
            #[singleton(implementation = MySql)]
            db: Arc<Db>,

            #[singleton]
            replica: $field,
        }
    };
}

shop!(Arc<Db>, MySql);
