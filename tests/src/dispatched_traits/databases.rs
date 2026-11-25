//! A trait with every kind of method static dispatch forwards, its marked
//! supertrait, and three implementations.

use std::fmt::{Debug, Display};

use snapfit::__private::{Either, Only};

/// Every database on the list, as a value names it.
pub type Db = Either<MySql, Either<Postgres, Only<Mock>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: u32,
    pub source: &'static str,
}

#[snapfit::static_dispatch]
pub trait Named {
    fn name(&self) -> &'static str;
}

/// Used only inside this crate, so `async fn` needs no `Send` bound.
#[allow(async_fn_in_trait)]
#[snapfit::static_dispatch]
pub trait Database: Named + Clone + Debug + Send + Sync {
    type Row;

    fn find(&self, id: u32) -> Option<Self::Row>;

    fn tagged<T: Display>(&self, tag: T) -> String;

    fn shown(&self, value: impl Display) -> String;

    fn store(&mut self, id: u32);

    fn stored(&self) -> Vec<u32>;

    fn into_name(self) -> String
    where
        Self: Sized;

    fn renamed(self, prefix: &'static str) -> Self
    where
        Self: Sized;

    async fn ping(&self) -> String;

    fn greeting(&self) -> String {
        format!("hello from {}", self.name())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MySql {
    pub stored: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct Postgres {
    pub stored: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct Mock {
    pub stored: Vec<u32>,
    pub prefix: &'static str,
}

/// The same impl for each database, which differs only by its name.
macro_rules! database {
    ($type:ty, $name:literal) => {
        impl Named for $type {
            fn name(&self) -> &'static str {
                $name
            }
        }

        impl Database for $type {
            type Row = Order;

            fn find(&self, id: u32) -> Option<Order> {
                Some(Order { id, source: $name })
            }

            fn tagged<T: Display>(&self, tag: T) -> String {
                format!("{}:{tag}", $name)
            }

            fn shown(&self, value: impl Display) -> String {
                format!("{} shows {value}", $name)
            }

            fn store(&mut self, id: u32) {
                self.stored.push(id);
            }

            fn stored(&self) -> Vec<u32> {
                self.stored.clone()
            }

            fn into_name(self) -> String {
                $name.to_owned()
            }

            fn renamed(self, _prefix: &'static str) -> Self {
                self
            }

            async fn ping(&self) -> String {
                format!("{} answers", $name)
            }
        }
    };
}

database!(MySql, "mysql");
database!(Postgres, "postgres");

impl Named for Mock {
    fn name(&self) -> &'static str {
        "mock"
    }
}

/// The mock overrides the default greeting, and keeps its prefix in a rename.
impl Database for Mock {
    type Row = Order;

    fn find(&self, id: u32) -> Option<Order> {
        Some(Order { id, source: "mock" })
    }

    fn tagged<T: Display>(&self, tag: T) -> String {
        format!("mock:{tag}")
    }

    fn shown(&self, value: impl Display) -> String {
        format!("mock shows {value}")
    }

    fn store(&mut self, id: u32) {
        self.stored.push(id);
    }

    fn stored(&self) -> Vec<u32> {
        self.stored.clone()
    }

    fn into_name(self) -> String {
        format!("{}mock", self.prefix)
    }

    fn renamed(self, prefix: &'static str) -> Self {
        Self { prefix, ..self }
    }

    async fn ping(&self) -> String {
        "mock answers".to_owned()
    }

    fn greeting(&self) -> String {
        "the mock greets".to_owned()
    }
}
