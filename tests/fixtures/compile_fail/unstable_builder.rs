use std::sync::Arc;
use snapfit::container;

pub struct Config;
pub struct AuthService;

#[container]
pub struct BuilderContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [config], builder = build_auth)]
    auth: Arc<AuthService>,
}
