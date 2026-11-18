//! Containers whose fields are built by the factories the user writes.

/// Two `Arc<Database>` fields passed to one constructor, which a factory
/// receives by name, so that they cannot land in each other's place.
pub mod orders {
    use std::sync::Arc;

    use snapfit::container;

    use crate::services::{Config, Database};

    pub trait OrderStore: Send + Sync {
        fn reader(&self) -> &Arc<Database>;
        fn writer(&self) -> &Arc<Database>;
    }

    pub struct SqlOrderStore {
        reader: Arc<Database>,
        writer: Arc<Database>,
    }

    impl SqlOrderStore {
        pub fn new(reader: Arc<Database>, writer: Arc<Database>) -> Self {
            Self { reader, writer }
        }
    }

    impl OrderStore for SqlOrderStore {
        fn reader(&self) -> &Arc<Database> {
            &self.reader
        }

        fn writer(&self) -> &Arc<Database> {
            &self.writer
        }
    }

    #[container]
    pub struct Shop {
        #[singleton]
        config: Arc<Config>,

        #[singleton(deps = [config])]
        read_db: Arc<Database>,

        #[singleton(deps = [config])]
        write_db: Arc<Database>,

        #[singleton(deps = [write_db, read_db], build = factory)]
        order_store: Arc<dyn OrderStore>,
    }

    impl ShopFactories for Shop {
        fn build_order_store(inputs: ShopOrderStoreInputs) -> Arc<dyn OrderStore> {
            let ShopOrderStoreInputs { read_db, write_db } = inputs;
            Arc::new(SqlOrderStore::new(read_db, write_db))
        }
    }
}

/// A trait-object field built by its factory, and a service whose
/// constructor takes that trait object.
pub mod payments {
    use std::sync::Arc;

    use snapfit::container;

    pub trait PaymentGateway: Send + Sync {
        fn charge(&self, cents: u32) -> String;
    }

    pub struct SandboxGateway {
        merchant: String,
    }

    impl PaymentGateway for SandboxGateway {
        fn charge(&self, cents: u32) -> String {
            format!("{} charged {cents} cents in the sandbox", self.merchant)
        }
    }

    pub struct Checkout {
        gateway: Arc<dyn PaymentGateway>,
    }

    impl Checkout {
        pub fn new(gateway: Arc<dyn PaymentGateway>) -> Self {
            Self { gateway }
        }

        pub fn gateway(&self) -> &Arc<dyn PaymentGateway> {
            &self.gateway
        }
    }

    #[container]
    pub struct Store {
        #[singleton(build = factory)]
        payments: Arc<dyn PaymentGateway>,

        #[singleton(deps = [payments])]
        checkout: Arc<Checkout>,
    }

    impl StoreFactories for Store {
        fn build_payments(_inputs: StorePaymentsInputs) -> Arc<dyn PaymentGateway> {
            Arc::new(SandboxGateway {
                merchant: "northwind".to_string(),
            })
        }
    }
}
