# wireup (`wireup-rs`)

> **Zero-Cost, Compile-Time Verified, Pure-Rust Dependency Injection Framework**

[![Crates.io](https://img.shields.io/badge/crates.io-wireup-orange)](https://crates.io/crates/wireup)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#)

---

## What is `wireup`?

`wireup` is a modern, compile-time verified dependency injection framework designed specifically for idiomatic Rust. 

Unlike traditional DI frameworks ported from Java (Spring/Dagger) or C# (.NET), `wireup` inverts the dependency declaration: **the container struct declares the wiring, leaving your domain structs 100% pure Rust.**

```text
Traditional DI:  Pollutes every service with #[inject], #[component], and dyn Any.
wireup:          Domain types stay pure Rust. The container wires them at compile time.
```

---

## Quick Look

```rust
use std::sync::Arc;
use wireup::Container;

// 1. Your domain types are 100% pure Rust (ZERO macros, ZERO framework traits!)
pub struct UserController {
    config: Arc<Config>,
    db: Arc<Database>,
}

impl UserController {
    pub fn new(config: Arc<Config>, db: Arc<Database>) -> Self {
        Self { config, db }
    }
}

// 2. The container declares the dependency graph in one place:
#[derive(Container)]
#[container(mode = "async")]
pub struct AppContainer {
    #[dep]
    config: Arc<Config>,

    // Async constructor: Database::connect(config).await
    #[dep(config, async, constructor = "connect")]
    db: Arc<Database>,

    // Transient: freshly constructed on every call
    #[dep(config, db, transient)]
    user_controller: UserController,
}

// 3. Eager async boot with instant, synchronous getters:
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // All async connections resolve topologically at startup:
    let container = AppContainer::init().await?;

    // Runtime getters are 100% synchronous, non-blocking, and lock-free!
    let controller = container.get_user_controller();
    
    Ok(())
}
```

---

## Key Highlights

* **Zero Domain Pollution:** Your business logic, controllers, and services require **no macros or framework traits**. They are plain Rust structs with standard `pub fn new(...)` constructors.
* **Compile-Time DAG Verification:** Missing prerequisites and circular dependencies (e.g. `A -> B -> A`) are caught during `cargo build` with clear compiler errors.
* **Zero `dyn Any` Overhead:** Direct, strongly-typed getters (`get_<dep>()`) with zero runtime downcasting.
* **Static Trait Polymorphism:** Automatically generates `enum_dispatch` enums for traits (`#[dep(trait = Database, impls = [PgDb, MockDb])]`), giving trait abstraction with zero vtables and zero heap indirection.
* **Fork-and-Rewire Testing:** Override any dependency with a mock (`container.fork_with().with_db(mock).build()`). Downstream dependents are automatically re-instantiated.
* **Request-Scoped Child Containers:** Lightweight child containers (`#[container(parent = AppContainer)]`) allow request data (like `UserSession`) to borrow application singletons without re-creating them.

---

## Documentation

* [Specification & API Guide](docs/SPEC.md) — Comprehensive functional specification, attribute syntax, and lifecycles.
* [Technical Architecture (Hexagonal)](docs/TECHNICAL_SPEC.md) — Ports & Adapters blueprint, Intermediate Representation (IR), and compiler isolation.
* [Development Workflow](docs/WORKFLOW.md) — Stacked PR roadmap, Double-Loop BDD, and agentic review loops.
* [Agent Guidelines](AGENTS.md) — Authoritative rules and architectural invariants for AI agents working in this repository.

---

## License

Dual-licensed under either of:
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
