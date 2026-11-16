use std::sync::Arc;

use snapfit::container;

use crate::services::{Config, Database, UserService};

#[container]
pub struct AppContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [config])]
    db: Arc<Database>,

    #[singleton(deps = [db])]
    user_service: Arc<UserService>,
}

pub mod multi_db {
    use std::sync::Arc;

    use snapfit::container;

    use crate::services::{Config, Database};

    #[container]
    pub struct AppContainer {
        #[singleton]
        config: Arc<Config>,

        #[singleton(deps = [config])]
        read_db: Arc<Database>,

        #[singleton(deps = [config])]
        write_db: Arc<Database>,
    }
}
