//! Two implementations, in a module of their own.

use crate::db::{Database, Store};
use crate::model::{Order, QueryError};

#[derive(Clone, Debug)]
pub struct Postgres {
    pub url: String,
}

#[derive(Clone, Debug, Default)]
pub struct Mock;

impl Database for Postgres {
    type Row = String;
    fn name(&self) -> &'static str { "postgres" }
    fn find(&self, id: u32) -> Result<Order, QueryError> { Ok(Order { id }) }
    fn rows(&self) -> Vec<String> { vec![self.url.clone()] }
    fn tagged<T: std::fmt::Display>(&self, tag: T) -> String { format!("pg:{tag}") }
    fn store(&mut self, order: Order) { self.url = format!("stored {}", order.id) }
    fn into_name(self) -> String { self.url }
    async fn ping(&self) -> bool { true }
}

impl Database for Mock {
    type Row = String;
    fn name(&self) -> &'static str { "mock" }
    fn find(&self, _id: u32) -> Result<Order, QueryError> { Err(QueryError) }
    fn rows(&self) -> Vec<String> { Vec::new() }
    fn tagged<T: std::fmt::Display>(&self, tag: T) -> String { format!("mock:{tag}") }
    fn store(&mut self, _order: Order) {}
    fn into_name(self) -> String { "mock".into() }
    async fn ping(&self) -> bool { false }
    fn with_default(&self) -> String { "mock overrides the default".into() }
}

impl Store<u8> for Postgres { fn put(&self, item: u8) -> usize { item as usize } }
impl Store<u8> for Mock { fn put(&self, _item: u8) -> usize { 0 } }
