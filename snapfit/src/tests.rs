use std::sync::Arc;

use crate::{Container, container};

#[derive(Debug, Default)]
struct Config {
    port: u16,
}

impl Config {
    fn new() -> Self {
        Self { port: 8080 }
    }
}

#[derive(Debug)]
struct Database {
    config: Arc<Config>,
}

impl Database {
    fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

#[derive(Debug)]
struct UserService {
    db: Arc<Database>,
}

impl UserService {
    fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[container]
pub struct ReverseOrderContainer {
    #[singleton(deps = [db])]
    user_service: Arc<UserService>,

    #[singleton(deps = [config])]
    db: Arc<Database>,

    #[singleton]
    config: Arc<Config>,
}

#[container]
pub(crate) struct TraitAppContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [config])]
    db: Arc<Database>,
}

#[test]
fn test_reverse_declaration_order_resolves_topologically() {
    let container = ReverseOrderContainer::new();
    let config = container.get_config();
    let db = container.get_db();
    let service = container.get_user_service();

    assert_eq!(config.port, 8080);
    assert!(Arc::ptr_eq(&config, &db.config));
    assert!(Arc::ptr_eq(&db, &service.db));
}

#[test]
fn test_container_trait_builds_a_container() {
    let container = <TraitAppContainer as Container>::new();
    let config = container.get_config();
    let db = container.get_db();

    assert_eq!(config.port, 8080);
    assert!(Arc::ptr_eq(&config, &db.config));
}
