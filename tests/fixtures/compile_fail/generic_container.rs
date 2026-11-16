use snapfit::container;

#[container]
pub struct Repo<T: Clone> {
    #[singleton]
    store: T,
}
