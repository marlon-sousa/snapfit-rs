<!-- Written by `cargo xtask crate-readme` from the README at the repository's root. Edit that one, then run it again. -->

# snapfit (`snapfit-rs`)

> **Zero-Cost, Compile-Time Verified, Pure-Rust Dependency Injection Framework**

Its source, its issues, and how to contribute are in [its repository on GitHub](https://github.com/marlon-sousa/snapfit-rs).


---

## What is `snapfit`?

`snapfit` is a compile-time verified dependency injection framework designed specifically for idiomatic Rust.

Unlike dependency injection frameworks ported from Java (Spring, Dagger) or C# (.NET) that scatter `#[inject]` and `#[component]` attributes across your business logic, or dynamic service locators that rely on `TypeMap` and `dyn Any` downcasting at runtime, `snapfit` inverts the dependency declaration: **the container struct declares the wiring, leaving your domain structs 100% pure Rust.**

---

## Why snapfit?

Most Rust applications eventually face three unsatisfying choices for wiring components:

1. **Invasive component macros (`#[inject]`, `#[component]`):** Every service, repository, and controller has to import DI macros and derive framework traits. Your domain is coupled to the DI framework, and seeing the whole dependency graph requires opening dozens of files.
2. **Dynamic `TypeMap` service locators (`Arc<dyn Any + Send + Sync>`):** Missing dependencies fail at runtime, resolution pays lookup and downcasting overhead, and registering two instances of the same type (`read_db: Database` and `write_db: Database`) silently overwrites the first unless you invent wrapper newtypes.
3. **Hand-rolled `main.rs` initialization chains:** Dozens of manual `Arc::clone` calls obscure which components are shared singletons and what depends on what.

| Guarantee | Traditional Component DI | Dynamic `TypeMap` Locators | `snapfit` |
| :--- | :--- | :--- | :--- |
| **Domain pollution** | Macros and traits on every service | Services or handlers pull from locator | **Zero**: domain structs are plain Rust |
| **Where the graph lives** | Scattered across dozens of files | Hidden inside runtime registration calls | **One central `#[container]` struct** |
| **Multiple instances of one type** | Requires qualifier macros on structs | Overwrites previous entry unless wrapped in newtypes | **Native**: edges name container fields (`deps = [read_db, write_db]`) |
| **Cycle & missing-dep detection** | Build script or runtime failure | Panics or returns `None` at runtime | **Compile-time**, with the error on the offending field |
| **Resolution cost** | Vtable or generated factory | Hash lookup + `dyn Any` downcast | **Zero-cost** direct getters (`get_<name>()`) |

---

## Releases and The Snapfit Book

**The Snapfit Book** walks through the framework step by step with complete, compiled Rust programs for each release.

Each release's edition of the book is on [the snapfit site](https://marlon-sousa.github.io/snapfit-rs/releases/).


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
    #[singleton(deps = [config], build = constructor(connect))]
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
* **Compile-time graph checks:** unknown dependencies and cycles, such as `service_a` depending on `service_b` which depends on `service_a`, fail `cargo build`, each reported at its field and all of them in one compile. Fields are visited in the order they are declared, and the fields each one needs are built before it.
* **Typed getters:** `get_<name>()` returns the declared type with no `dyn Any` and no forced `Arc<T>` (any `T: Clone` works).
* **Named edges:** two dependencies of the same type (`read_db` and `write_db`) are wired by field name, without wrapper newtypes.
* **Named constructors:** `build = constructor(connect)` calls `Type::connect(...)` instead of `Type::new(...)`.
* **Defaults:** `build = default` calls `Default::default()` for a type that only implements `Default`, and `#[container(leaf_build = default)]` does it for every field without `deps` or a `build` of its own.
* **Factories:** `build = factory` has the container call a `build_<name>` method you write in the generated `<Container>Factories` trait, which receives the field's `deps` by name in one struct; it builds trait-object fields and anything a plain constructor cannot.
* **Builders:** `build = builder` calls `Type::builder()`, passes each of the field's `deps` to the setter of its name, and finishes with `.build()`. A label, `deps = [disk = folder_files]`, passes a dependency to a setter with another name, and `build = builder(start = draft, finish = publish)` names other functions. Any builder works, written by hand or generated by a crate such as `bon` or `typed-builder`; with a typestate builder, a missing setter is a compile error too.


---

## Minimum Supported Rust Version (MSRV)

This crate requires **Rust 1.85.0** or higher (Rust 2024 edition), which CI checks.


---

## License

Dual-licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/marlon-sousa/snapfit-rs/blob/main/LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](https://github.com/marlon-sousa/snapfit-rs/blob/main/LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.
