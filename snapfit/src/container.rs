use std::future::Future;

pub trait Container: Sized {
    fn new() -> Self;
}

pub trait AsyncContainer: Sized {
    type Error: std::error::Error + Send + Sync + 'static;

    fn init() -> impl Future<Output = Result<Self, Self::Error>> + Send;
}
