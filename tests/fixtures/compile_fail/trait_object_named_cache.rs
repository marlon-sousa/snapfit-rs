use snapfit::container;

pub trait Cache {}

#[container]
pub struct CacheContainer {
    #[singleton]
    cache: Box<dyn Cache>,
}
