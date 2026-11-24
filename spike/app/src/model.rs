//! Types the trait's methods name, which the container's module never imports.

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u32,
}

#[derive(Debug)]
pub struct QueryError;
