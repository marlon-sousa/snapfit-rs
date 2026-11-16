use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct Database;
pub struct UserService;

#[container]
pub struct BadContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [confog])]
    db: Arc<Database>,

    #[singleton(deps = [db])]
    user_service: Arc<UserService>,
}
