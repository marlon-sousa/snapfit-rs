use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Config {
    pub app_name: &'static str,
}

impl Config {
    pub fn new() -> Self {
        Self {
            app_name: "snapfit",
        }
    }
}

#[derive(Debug)]
pub struct Database {
    config: Arc<Config>,
}

impl Database {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub fn get_config_ref(&self) -> Arc<Config> {
        Arc::clone(&self.config)
    }
}

#[derive(Debug)]
pub struct UserService {
    db: Arc<Database>,
}

impl UserService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn get_db_ref(&self) -> Arc<Database> {
        Arc::clone(&self.db)
    }
}
