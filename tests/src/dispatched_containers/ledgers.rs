//! A trait marked for static dispatch, its implementations, and a service
//! that takes the type a container dispatches it as.

use std::sync::Arc;

use super::accounts::Ledgers;

#[snapfit::static_dispatch]
pub trait Ledger {
    fn source(&self) -> String;
}

pub struct Config {
    pub host: &'static str,
}

impl Config {
    pub fn new() -> Self {
        Self {
            host: "ledger.example",
        }
    }

    pub fn offline() -> Self {
        Self { host: "" }
    }
}

pub struct Disk;

impl Disk {
    pub fn new() -> Self {
        Self
    }
}

pub struct Remote {
    host: &'static str,
}

impl Remote {
    pub fn connect(config: Arc<Config>) -> Self {
        Self { host: config.host }
    }

    /// Fails when the host is empty, as a connection that cannot open would.
    pub fn try_connect(config: Arc<Config>) -> Result<Self, String> {
        match config.host {
            "" => Err("no host to connect to".to_owned()),
            host => Ok(Self { host }),
        }
    }
}

#[derive(Default)]
pub struct Memory;

/// Built by no field, only by a test, added by an extension with no condition.
pub struct Fake;

impl Ledger for Disk {
    fn source(&self) -> String {
        "disk".to_owned()
    }
}

impl Ledger for Remote {
    fn source(&self) -> String {
        format!("remote at {}", self.host)
    }
}

impl Ledger for Memory {
    fn source(&self) -> String {
        "memory".to_owned()
    }
}

impl Ledger for Fake {
    fn source(&self) -> String {
        "fake".to_owned()
    }
}

/// Takes the dispatched type, and calls the trait on it without a match of its own.
pub struct Books {
    ledger: Arc<Ledgers>,
}

impl Books {
    pub fn new(ledger: Arc<Ledgers>) -> Self {
        Self { ledger }
    }

    pub fn source(&self) -> String {
        self.ledger.source()
    }
}
