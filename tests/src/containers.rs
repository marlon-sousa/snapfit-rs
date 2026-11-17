use std::sync::Arc;

use snapfit::container;

use crate::services::{Config, Database, UserService};

#[container]
pub struct AppContainer {
    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [config])]
    db: Arc<Database>,

    #[singleton(deps = [db])]
    user_service: Arc<UserService>,
}

pub mod multi_db {
    use std::sync::Arc;

    use snapfit::container;

    use crate::services::{Config, Database};

    #[container]
    pub struct AppContainer {
        #[singleton]
        config: Arc<Config>,

        #[singleton(deps = [config])]
        read_db: Arc<Database>,

        #[singleton(deps = [config])]
        write_db: Arc<Database>,
    }
}

/// Each value records its name when it is built, so a scenario can read the
/// order the container's constructors run in.
pub mod ordered {
    use std::cell::RefCell;

    use snapfit::container;

    thread_local! {
        static BUILT: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    }

    fn record(name: &'static str) {
        BUILT.with_borrow_mut(|built| built.push(name));
    }

    /// The names recorded since the last call, in the order they were built.
    pub fn take_built() -> Vec<&'static str> {
        BUILT.with_borrow_mut(std::mem::take)
    }

    #[derive(Clone)]
    pub struct Config;
    #[derive(Clone)]
    pub struct Db;
    #[derive(Clone)]
    pub struct Service;
    #[derive(Clone)]
    pub struct Clock;

    impl Config {
        pub fn new() -> Self {
            record("config");
            Self
        }
    }

    impl Db {
        pub fn new(_config: Config) -> Self {
            record("db");
            Self
        }
    }

    impl Service {
        pub fn new(_db: Db) -> Self {
            record("service");
            Self
        }
    }

    impl Clock {
        pub fn new() -> Self {
            record("clock");
            Self
        }
    }

    #[container]
    pub struct OrderedContainer {
        #[singleton(deps = [db])]
        service: Service,

        #[singleton(deps = [config])]
        db: Db,

        #[singleton]
        config: Config,

        #[singleton]
        clock: Clock,
    }
}

/// The `payments` field holds the concrete gateway, while the service's
/// constructor takes the trait object: Rust converts the `Arc` where the
/// generated code passes it.
pub mod checkout {
    use std::sync::Arc;

    use snapfit::container;

    pub trait PaymentGateway: Send + Sync {
        fn charge(&self, cents: u32) -> String;
    }

    #[derive(Default)]
    pub struct StripeGateway;

    impl StripeGateway {
        pub fn new() -> Self {
            Self
        }
    }

    impl PaymentGateway for StripeGateway {
        fn charge(&self, cents: u32) -> String {
            format!("stripe charged {cents} cents")
        }
    }

    pub struct CheckoutService {
        gateway: Arc<dyn PaymentGateway>,
    }

    impl CheckoutService {
        pub fn new(gateway: Arc<dyn PaymentGateway>) -> Self {
            Self { gateway }
        }

        pub fn gateway(&self) -> &Arc<dyn PaymentGateway> {
            &self.gateway
        }
    }

    #[container]
    pub struct CheckoutContainer {
        #[singleton]
        payments: Arc<StripeGateway>,

        #[singleton(deps = [payments])]
        checkout: Arc<CheckoutService>,
    }
}
