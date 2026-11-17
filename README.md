# snapfit (`snapfit-rs`)

> **Zero-Cost, Compile-Time Verified, Pure-Rust Dependency Injection Framework**

[![Crates.io](https://img.shields.io/badge/crates.io-snapfit-orange)](https://crates.io/crates/snapfit)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.85.0-informational?logo=rust)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#)

---

## What is `snapfit`?

`snapfit` is a modern, compile-time verified dependency injection framework designed specifically for idiomatic Rust. 

Unlike traditional DI frameworks ported from Java (Spring/Dagger) or C# (.NET), `snapfit` inverts the dependency declaration: **the container struct declares the wiring, leaving your domain structs 100% pure Rust.**

```text
Traditional DI:  Pollutes every service with #[inject], #[component], and dyn Any.
snapfit:         Domain types stay pure Rust. The container wires them at compile time.
```

---

## Quick Look

```rust
use std::sync::Arc;
use snapfit::container;

// 1. Your domain types are plain Rust: no macros, no framework traits.
pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn new() -> Self {
        Self { database_url: "postgres://localhost/shop".to_string() }
    }
}

pub struct Database {
    pub url: String,
}

impl Database {
    pub fn connect(config: Arc<Config>) -> Self {
        Self { url: config.database_url.clone() }
    }
}

pub struct UserService {
    db: Arc<Database>,
}

impl UserService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

// 2. The container declares the dependency graph in one place:
#[container]
pub struct AppContainer {
    // Leaf singleton: calls Config::new()
    #[singleton]
    config: Arc<Config>,

    // Named constructor: calls Database::connect(config)
    #[singleton(deps = [config], constructor = connect)]
    db: Arc<Database>,

    // Calls UserService::new(db)
    #[singleton(deps = [db])]
    users: Arc<UserService>,
}

// 3. Build once, then use the typed getters:
fn main() {
    let container = AppContainer::new();
    let users = container.get_users();
    assert!(Arc::ptr_eq(&container.get_db(), &users.db));
}
```

---

## What 0.1.0 Supports

* **Plain domain types:** services, repositories and controllers need no macros or framework traits.
* **Compile-time graph checks:** unknown dependencies and cycles, such as `service_a` depending on `service_b` which depends on `service_a`, fail `cargo build`, each reported at its field and all of them in one compile. Fields are built in the order they are declared, each after the fields it needs.
* **Typed getters:** `get_<name>()` returns the declared type with no `dyn Any` and no forced `Arc<T>` (any `T: Clone` works).
* **Named edges:** two dependencies of the same type (`read_db` and `write_db`) are wired by field name, without wrapper newtypes.
* **Named constructors:** `constructor = connect` calls `Type::connect(...)` instead of `Type::new(...)`.

### Unstable APIs (`features = ["unstable"]`)

These work today but their API may still change, so they need the `unstable` feature of the `snapfit` crate:

* **Async containers:** `#[container(mode = "async")]`, `#[singleton(async, ...)]` and `AppContainer::init().await`, with synchronous getters afterwards.
* **Transient dependencies:** `#[transient]` fields are not stored and are built fresh on every `get_<name>()`.
* **Custom builders:** `builder = build_auth` calls an associated function you write on the container.

### Planned

Trait dispatch (`#[dispatch(...)]`), test overrides (`fork_with()`), request-scoped child containers (`#[container(parent = ...)]`) and graph introspection (`introspection = true`) are designed in [docs/SPEC.md](docs/SPEC.md) but not implemented; using them is a compile error. See [What ships in 0.1.0](docs/SPEC.md#0-what-ships-in-010).

---

## Minimum Supported Rust Version (MSRV)

This crate requires **Rust 1.85.0** or higher (Rust 2024 edition).
The exact toolchain and required components (`clippy`, `rustfmt`, `rust-src`) are pinned in [`rust-toolchain.toml`](rust-toolchain.toml).

---

## Documentation

* [Specification & API Guide](docs/SPEC.md) — Comprehensive functional specification, attribute syntax, and lifecycles.
* [Architecture](docs/ARCHITECTURE.md) — how the code is organised and written: module roles, the typed model, errors and tests.
* [Development Workflow](docs/WORKFLOW.md) — Stacked PR roadmap, Double-Loop BDD, and agentic review loops.
* [Agent Guidelines](AGENTS.md) — Authoritative rules, reuse guidelines, and architectural invariants for AI agents working in this repository.

---

## License

Dual-licensed under either of:
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
