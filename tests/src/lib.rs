#![deny(warnings)]

pub mod compile_harness;
pub mod containers;
pub mod services;

pub use compile_harness::{CompileOutcome, compile_fixture};
pub use containers::{AppContainer, multi_db};
pub use services::{Config, Database, UserService};
