//! A container whose factories choose which implementation its dispatched
//! fields hold, one in an `Arc` and one as a value, from a setting read while
//! the program starts. Each variant needs inputs of its own, and the factory
//! receives the inputs of every variant.

use std::cell::Cell;
use std::sync::Arc;

use cucumber::Parameter;
use snapfit::container;
use strum::EnumString;

use super::ledgers::{Config, Ledger};

/// The implementation a setting names, as a scenario writes it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, EnumString, Parameter)]
#[strum(serialize_all = "lowercase")]
#[param(name = "backend", regex = "archive|mirror|scratch")]
pub enum Backend {
    #[default]
    Archive,
    Mirror,
    Scratch,
}

thread_local! {
    /// Stands for an environment variable, which a scenario cannot set
    /// without touching every other scenario that runs beside it.
    static BACKEND: Cell<Backend> = Cell::default();
}

/// The setting the factories choose from, read when the container is built.
pub struct Setting {
    pub backend: Backend,
}

impl Setting {
    pub fn new() -> Self {
        Self {
            backend: BACKEND.get(),
        }
    }
}

/// The folder only `Archive` reads.
pub struct Folder {
    pub path: &'static str,
}

impl Folder {
    pub fn new() -> Self {
        Self { path: "ledgers" }
    }
}

#[derive(Clone)]
pub struct Archive {
    path: &'static str,
}

/// Reads its host from the `Config` only it needs.
#[derive(Clone)]
pub struct Mirror {
    host: &'static str,
}

/// Needs no input at all.
#[derive(Clone)]
pub struct Scratch;

impl Ledger for Archive {
    fn source(&self) -> String {
        format!("archive in {}", self.path)
    }
}

impl Ledger for Mirror {
    fn source(&self) -> String {
        format!("mirror of {}", self.host)
    }
}

impl Ledger for Scratch {
    fn source(&self) -> String {
        "scratch".to_owned()
    }
}

#[container]
#[static_dispatch_for(Ledger as Chosen, extra = [Archive, Mirror, Scratch])]
pub struct ChosenAccounts {
    #[singleton]
    setting: Arc<Setting>,

    #[singleton]
    folder: Arc<Folder>,

    #[singleton]
    config: Arc<Config>,

    #[singleton(deps = [setting, folder, config], build = factory)]
    shared: Arc<Chosen>,

    #[singleton(deps = [setting, folder, config], build = factory)]
    owned: Chosen,
}

impl ChosenAccountsFactories for ChosenAccounts {
    fn build_shared(inputs: ChosenAccountsSharedInputs) -> Arc<Chosen> {
        let ChosenAccountsSharedInputs {
            setting,
            folder,
            config,
        } = inputs;
        Arc::new(choose(&setting, &folder, &config))
    }

    fn build_owned(inputs: ChosenAccountsOwnedInputs) -> Chosen {
        let ChosenAccountsOwnedInputs {
            setting,
            folder,
            config,
        } = inputs;
        choose(&setting, &folder, &config)
    }
}

/// Builds the variant the setting names, from the inputs that variant needs.
fn choose(setting: &Setting, folder: &Folder, config: &Config) -> Chosen {
    match setting.backend {
        Backend::Archive => Chosen::of(Archive { path: folder.path }),
        Backend::Mirror => Chosen::of(Mirror { host: config.host }),
        Backend::Scratch => Chosen::of(Scratch),
    }
}

/// Builds the container as a program would that starts with `backend` set.
pub fn start_with(backend: Backend) -> ChosenAccounts {
    BACKEND.set(backend);
    ChosenAccounts::new()
}
