#![deny(warnings)]
//! Chapter 10: dependencies passed by name, through a builder.

use snapfit::container;
use std::sync::Arc;

pub struct Database {
    role: &'static str,
}

impl Database {
    pub fn primary() -> Self {
        Self { role: "primary-rw" }
    }

    pub fn replica() -> Self {
        Self { role: "replica-ro" }
    }

    pub fn role(&self) -> &'static str {
        self.role
    }
}

// ANCHOR: sales_report
pub struct SalesReport {
    summary: String,
}

#[bon::bon]
impl SalesReport {
    #[builder(start_fn = builder)]
    pub fn new(live: Arc<Database>, ledger: Arc<Database>) -> Self {
        let summary = format!(
            "today's sales from {}, totals checked against {}",
            live.role(),
            ledger.role()
        );
        Self { summary }
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }
}
// ANCHOR_END: sales_report

// ANCHOR: container
#[container]
pub struct ShopContainer {
    #[singleton(build = constructor(replica))]
    read_db: Arc<Database>,

    #[singleton(build = constructor(primary))]
    write_db: Arc<Database>,

    #[singleton(deps = [ledger = write_db, live = read_db], build = builder)]
    sales_report: Arc<SalesReport>,
}
// ANCHOR_END: container

// ANCHOR: run
fn main() {
    let container = ShopContainer::new();
    println!("{}", container.get_sales_report().summary());
}
// ANCHOR_END: run
