use std::sync::Arc;
use snapfit::container;

pub struct AuthService;

#[container]
pub struct Shop {
    #[singleton(builder = build_auth)]
    auth: Arc<AuthService>,
}
