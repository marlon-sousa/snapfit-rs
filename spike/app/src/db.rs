//! The trait, marked dispatchable, with every kind of method the spike checks.

use crate::model::{Order, QueryError};

#[dispatchable::dispatchable]
pub trait Database: Send + Sync {
    type Row;

    fn name(&self) -> &'static str;
    fn find(&self, id: u32) -> Result<Order, QueryError>;
    fn rows(&self) -> Vec<Self::Row>;
    fn tagged<T: std::fmt::Display>(&self, tag: T) -> String;
    fn store(&mut self, order: Order);
    fn into_name(self) -> String
    where
        Self: Sized;
    async fn ping(&self) -> bool;
    fn with_default(&self) -> String {
        format!("default for {}", self.name())
    }
}

/// A generic trait, implemented by each variant for the same type argument.
#[dispatchable::dispatchable]
pub trait Store<T> {
    fn put(&self, item: T) -> usize;
}
