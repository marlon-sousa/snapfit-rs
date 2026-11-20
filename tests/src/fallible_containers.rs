//! Containers with fields marked `fallible = true`, built by constructors,
//! builders and factories that return a `Result`.

pub mod till;

/// Every way of building a field, each able to fail, and each succeeding,
/// in a container whose leaf_build cannot build its fallible leaf, which
/// therefore names its own build.
pub mod opening {
    use std::error::Error;
    use std::fmt;
    use std::io;
    use std::sync::Arc;

    use snapfit::container;

    #[derive(Debug)]
    pub struct ConfigError;

    impl fmt::Display for ConfigError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("the shop's configuration is missing")
        }
    }

    impl Error for ConfigError {}

    pub struct ShopConfig {
        pub currency: &'static str,
    }

    impl ShopConfig {
        pub fn load() -> Result<Self, ConfigError> {
            Ok(Self { currency: "EUR" })
        }
    }

    #[derive(Default)]
    pub struct ReceiptPrinter {
        pub paper: &'static str,
    }

    pub struct TaxRates {
        pub standard_percent: u8,
    }

    impl TaxRates {
        pub fn new() -> Result<Self, String> {
            Ok(Self {
                standard_percent: 20,
            })
        }
    }

    pub struct Database {
        pub config: Arc<ShopConfig>,
    }

    #[bon::bon]
    impl Database {
        #[builder]
        pub fn new(config: Arc<ShopConfig>) -> Result<Self, io::Error> {
            Ok(Self { config })
        }
    }

    pub trait PaymentGateway: Send + Sync {
        fn name(&self) -> &'static str;
    }

    pub struct Stripe;

    impl PaymentGateway for Stripe {
        fn name(&self) -> &'static str {
            "stripe"
        }
    }

    pub struct OrderService {
        pub db: Arc<Database>,
        pub payments: Arc<dyn PaymentGateway>,
    }

    impl OrderService {
        pub fn new(db: Arc<Database>, payments: Arc<dyn PaymentGateway>) -> Self {
            Self { db, payments }
        }
    }

    #[container(leaf_build = default)]
    pub struct Shop {
        #[singleton(fallible = false)]
        receipts: Arc<ReceiptPrinter>,

        #[singleton(fallible = true, build = constructor(load))]
        config: Arc<ShopConfig>,

        #[singleton(fallible = true, build = constructor(new))]
        rates: Arc<TaxRates>,

        #[singleton(deps = [config], fallible = true, build = builder)]
        db: Arc<Database>,

        #[singleton(fallible = true, build = factory)]
        payments: Arc<dyn PaymentGateway>,

        #[singleton(deps = [db, payments])]
        orders: Arc<OrderService>,
    }

    impl ShopFactories for Shop {
        fn build_payments(
            _inputs: ShopPaymentsInputs,
        ) -> Result<Arc<dyn PaymentGateway>, Box<dyn Error + Send + Sync>> {
            Ok(Arc::new(Stripe))
        }
    }
}

/// A shop whose database fails to connect, after its configuration was
/// built and before its order service is.
pub mod outage {
    use std::cell::RefCell;
    use std::error::Error;
    use std::fmt;
    use std::sync::Arc;

    use snapfit::container;

    thread_local! {
        static EVENTS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    }

    fn record(event: &'static str) {
        EVENTS.with_borrow_mut(|events| events.push(event));
    }

    /// The events recorded since the last call, in the order they happened.
    pub fn take_events() -> Vec<&'static str> {
        EVENTS.with_borrow_mut(std::mem::take)
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct PortTaken(pub u16);

    impl fmt::Display for PortTaken {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "port {} is taken", self.0)
        }
    }

    impl Error for PortTaken {}

    pub struct Config;

    impl Config {
        pub fn new() -> Self {
            record("config built");
            Self
        }
    }

    impl Drop for Config {
        fn drop(&mut self) {
            record("config dropped");
        }
    }

    #[derive(Clone)]
    pub struct Database;

    impl Database {
        pub fn connect(_config: Arc<Config>) -> Result<Self, PortTaken> {
            record("database tried");
            Err(PortTaken(5432))
        }
    }

    #[derive(Clone)]
    pub struct OrderService;

    impl OrderService {
        pub fn new(_db: Database) -> Self {
            record("orders built");
            Self
        }
    }

    #[container]
    pub struct Storefront {
        #[singleton(deps = [db])]
        orders: OrderService,

        #[singleton(deps = [config], fallible = true, build = constructor(connect))]
        db: Database,

        #[singleton]
        config: Arc<Config>,
    }
}
