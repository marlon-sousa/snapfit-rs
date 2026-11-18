//! Containers whose fields say how they are built: by `Default`, or by a
//! named constructor, written by hand or by a `macro_rules!`.

/// A leaf built by `Default` and a database opened by a constructor that is
/// not `new`.
pub mod named {
    use std::sync::Arc;

    use snapfit::container;

    #[derive(Default)]
    pub struct TaxRates {
        pub standard_percent: u32,
    }

    pub struct Ledger {
        pub rates: Arc<TaxRates>,
        pub opened_by: &'static str,
    }

    impl Ledger {
        pub fn open(rates: Arc<TaxRates>) -> Self {
            Self {
                rates,
                opened_by: "open",
            }
        }
    }

    #[container]
    pub struct Books {
        #[singleton(build = default)]
        rates: Arc<TaxRates>,

        #[singleton(deps = [rates], build = constructor(open))]
        ledger: Arc<Ledger>,
    }
}

/// Leaves built by `Default` for the whole container, beside a leaf that
/// keeps its own `build` and a field with `deps`, which `leaf_build` leaves
/// to `new`.
pub mod leaves {
    use std::sync::Arc;

    use snapfit::container;

    #[derive(Default)]
    pub struct Settings {
        pub currency: &'static str,
    }

    #[derive(Default, Clone)]
    pub struct Clock;

    pub struct Mailer {
        pub sender: &'static str,
    }

    impl Mailer {
        pub fn sandbox() -> Self {
            Self { sender: "sandbox" }
        }
    }

    pub struct Invoicer {
        pub settings: Arc<Settings>,
    }

    impl Invoicer {
        pub fn new(settings: Arc<Settings>) -> Self {
            Self { settings }
        }
    }

    #[container(leaf_build = default)]
    pub struct Billing {
        #[singleton]
        settings: Arc<Settings>,

        #[singleton]
        clock: Clock,

        #[singleton(build = constructor(sandbox))]
        mailer: Arc<Mailer>,

        #[singleton(deps = [settings])]
        invoicer: Arc<Invoicer>,
    }
}

/// Leaves opened by a constructor the container names once, beside a leaf
/// whose own `build` wins.
pub mod archive {
    use snapfit::container;

    #[derive(Clone)]
    pub struct Shelf {
        pub opened_by: &'static str,
    }

    impl Shelf {
        pub fn open() -> Self {
            Self { opened_by: "open" }
        }
    }

    impl Default for Shelf {
        fn default() -> Self {
            Self {
                opened_by: "default",
            }
        }
    }

    #[container(leaf_build = constructor(open))]
    pub struct Archive {
        #[singleton]
        books: Shelf,

        #[singleton]
        maps: Shelf,

        #[singleton(build = default)]
        spare: Shelf,
    }
}

/// A container declared by a `macro_rules!` that passes its `leaf_build`, a
/// `deps` entry and a constructor's name as `$e:expr`, which wraps each in an
/// invisible group.
pub mod generated {
    use std::sync::Arc;

    use snapfit::container;

    #[derive(Default)]
    pub struct Settings {
        pub region: &'static str,
    }

    pub struct Mailer {
        pub settings: Arc<Settings>,
        pub opened_by: &'static str,
    }

    impl Mailer {
        pub fn connect(settings: Arc<Settings>) -> Self {
            Self {
                settings,
                opened_by: "connect",
            }
        }
    }

    macro_rules! office {
        ($leaf:expr, $dependency:expr, $function:expr) => {
            #[container(leaf_build = $leaf)]
            pub struct Office {
                #[singleton]
                settings: Arc<Settings>,

                #[singleton(deps = [$dependency], build = constructor($function))]
                mailer: Arc<Mailer>,
            }
        };
    }

    office!(default, settings, connect);
}
